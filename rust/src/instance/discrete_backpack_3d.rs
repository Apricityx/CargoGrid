#[derive(Clone, Debug)]
struct Object {
    name: usize,
    width: usize,
    length: usize,
    height: usize,
    value: usize,
}

struct Limit {
    width: usize,
    length: usize,
    height: usize,
}

struct Solve {
    limit: Limit,
    objects: Vec<Object>,
    // [length][width][height]，每个格子存 Option<usize>，表示第几个物体
    global_best_solution: Vec<Vec<Vec<Option<usize>>>>,
    global_best_select_objects: Vec<Object>,
    global_best_cost: usize,
    current_solution: Vec<Vec<Vec<Option<usize>>>>,
    current_selected_objects: Vec<Object>,
}

impl Solve {
    fn new(limit: Limit, objects: Vec<Object>) -> Self {
        let l = limit.length;
        let w = limit.width;
        let h = limit.height;

        let empty = vec![vec![vec![None; h]; w]; l];
        Solve {
            limit,
            objects,
            current_solution: empty.clone(),
            global_best_solution: empty,
            global_best_select_objects: vec![],
            global_best_cost: 0,
            current_selected_objects: vec![],
        }
    }

    /// 回溯搜索：尝试从 current_object_index 开始装物体
    fn dfs(&mut self, current_object_index: usize, prev_cost: usize) {
        // 1. 触底：所有物体都处理完了
        if current_object_index == self.objects.len() {
            if prev_cost > self.global_best_cost {
                self.global_best_cost = prev_cost;
                self.global_best_solution = self.current_solution.clone();
                self.global_best_select_objects = self.current_selected_objects.clone();
            }
            return;
        }

        let object_index = current_object_index;

        // 先把要用到的字段拷出来
        let (obj_len, obj_w, obj_h, obj_val) = {
            let obj = &self.objects[object_index];
            (obj.length, obj.width, obj.height, obj.value)
        };

        // 分支一：不装当前物体（无论怎样都可以选这条）
        self.dfs(current_object_index + 1, prev_cost);

        // ⭐ 关键：如果物体本身在某一维就比箱子大，根本不可能装进去，
        //    直接 return，别去算 max_x / max_y / max_z 了。
        if obj_len > self.limit.length
            || obj_w > self.limit.width
            || obj_h > self.limit.height
        {
            return;
        }

        // 分支二：尝试装当前物体（枚举所有可行位置）
        let max_x = self.limit.length - obj_len + 1;
        let max_y = self.limit.width - obj_w + 1;
        let max_z = self.limit.height - obj_h + 1;

        for x in 0..max_x {
            for y in 0..max_y {
                for z in 0..max_z {
                    if self.can_place_object(object_index, x, y, z) {
                        let obj_clone = self.objects[object_index].clone();

                        // 选择：放进去 + 记录已经选中的物体
                        self.place(object_index, x, y, z);
                        self.current_selected_objects.push(obj_clone);

                        // 递归处理下一个物体
                        self.dfs(current_object_index + 1, prev_cost + obj_val);

                        // 回溯：撤销本次选择
                        self.current_selected_objects.pop();
                        self.remove(object_index, x, y, z);
                    }
                }
            }
        }
    }

    /// 检查从 (x, y, z) 开始，第 index 个 object 这块体积是否都为空
    fn can_place_object(&self, object_index: usize, x: usize, y: usize, z: usize) -> bool {
        let object = &self.objects[object_index];
        for dx in 0..object.length {
            for dy in 0..object.width {
                for dz in 0..object.height {
                    // 注意：这里假设 current_solution: [length][width][height]
                    if self.current_solution[x + dx][y + dy][z + dz].is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// 真正把 index 对应的 object 放进 current_solution 里
    fn place(&mut self, object_index: usize, x: usize, y: usize, z: usize) {
        let object = &self.objects[object_index];
        for dx in 0..object.length {
            for dy in 0..object.width {
                for dz in 0..object.height {
                    self.current_solution[x + dx][y + dy][z + dz] = Some(object_index);
                }
            }
        }
    }

    /// 把刚刚放进去的 index 对应 object 撤销（对应 place 的反操作）
    fn remove(&mut self, object_index: usize, x: usize, y: usize, z: usize) {
        let object = &self.objects[object_index];
        for dx in 0..object.length {
            for dy in 0..object.width {
                for dz in 0..object.height {
                    self.current_solution[x + dx][y + dy][z + dz] = None;
                }
            }
        }
    }
}

pub fn run_demo() {
    // 1. 定义一个 4x4x2 的“集装箱”
    let limit = Limit {
        length: 4,
        width: 3,
        height: 3,
    };

    // 2. 定义几件示例货物
    let objects = vec![
        Object {
            name: 1,
            length: 2,
            width: 2,
            height: 1,
            value: 6,
        },
        Object {
            name: 2,
            length: 1,
            width: 3,
            height: 1,
            value: 5,
        },
        Object {
            name: 3,
            length: 1,
            width: 2,
            height: 2,
            value: 7,
        },
        Object {
            name: 4,
            length: 2,
            width: 1,
            height: 1,
            value: 3,
        },
    ];

    let mut solver = Solve::new(limit, objects);

    // 3. 运行回溯搜索
    solver.dfs(0, 0);

    // 4. 打印结果
    println!("Best total value: {}", solver.global_best_cost);
    println!("Selected objects (name, l, w, h, value):");
    for o in &solver.global_best_select_objects {
        println!(
            "  name={} l={} w={} h={} value={}",
            o.name, o.length, o.width, o.height, o.value
        );
    }

    // 5. 打印三维布局（按 z 层输出，每层一个 2D 切片）
    println!("\nLayout (per height layer z):");
    for z in 0..solver.limit.height {
        println!("Layer z = {}", z);
        for x in 0..solver.limit.length {
            for y in 0..solver.limit.width {
                match solver.global_best_solution[x][y][z] {
                    Some(idx) => {
                        let obj = &solver.objects[idx];
                        print!("{:2} ", obj.name)
                    }
                    None => print!(" . "),
                }
            }
            println!();
        }
        println!();
    }
}

use std::collections::{BTreeSet, HashMap};

/// 运行求解并返回：
/// names: 参与摆放的物体名字数组（顺序稳定）
/// grid_zxy: 三维网格 [z][x][y]；空=-1，非空=names 的索引
/// selected: 最优解中的物体列表
/// best: 最优价值
pub fn solve_to_grid_zxy(
    limit: Limit,
    objects: Vec<Object>,
) -> (Vec<usize>, Vec<Vec<Vec<isize>>>, Vec<Object>, usize) {
    // 先跑原来的搜索
    let mut solver = Solve::new(limit, objects);
    solver.dfs(0, 0);

    let l = solver.limit.length; // x
    let w = solver.limit.width;  // y
    let h = solver.limit.height; // z

    // 1) 找到最优解中实际用到的 object 索引，并做稳定排序
    let mut used_indices = BTreeSet::new(); // 自动升序
    for x in 0..l {
        for y in 0..w {
            for z in 0..h {
                if let Some(idx) = solver.global_best_solution[x][y][z] {
                    used_indices.insert(idx);
                }
            }
        }
    }

    // 2) 构建名字数组（与 used_indices 同序）
    let names: Vec<usize> = used_indices
        .iter()
        .map(|&idx| solver.objects[idx].name)
        .collect();

    // 3) 建立：object 原始 idx -> names 中的下标
    let mut idx_to_namepos: HashMap<usize, isize> = HashMap::new();
    for (pos, &idx) in used_indices.iter().enumerate() {
        idx_to_namepos.insert(idx, pos as isize);
    }

    // 4) 构建 [z][x][y] 的栅格，空位=-1，非空=names 的索引
    let mut grid_zxy = vec![vec![vec![-1isize; w]; l]; h];
    for x in 0..l {
        for y in 0..w {
            for z in 0..h {
                if let Some(idx) = solver.global_best_solution[x][y][z] {
                    if let Some(&pos) = idx_to_namepos.get(&idx) {
                        grid_zxy[z][x][y] = pos;
                    }
                }
            }
        }
    }

    (
        names,
        grid_zxy,
        solver.global_best_select_objects.clone(),
        solver.global_best_cost,
    )
}
