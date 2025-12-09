use std::collections::BTreeSet;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    /// 物体在 x 轴方向的长度（原 length）
    pub x: usize,
    /// 物体在 y 轴方向的长度（原 width）
    pub y: usize,
    /// 物体在 z 轴方向的长度（原 height）
    pub z: usize,
    pub value: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Limit {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[derive(Clone)]
struct Solve {
    limit: Limit,
    objects: Vec<Object>,
    // DFS 搜索顺序：按价值从大到小排列的物体索引
    order: Vec<usize>,
    // 后缀价值和：suffix_values[pos] = 从 pos 开始理论可获得的最大剩余价值上界
    suffix_values: Vec<usize>,
    // [x][y][z]，每个格子存 Option<usize>，表示第几个物体（原始 objects 的下标）
    global_best_solution: Vec<Vec<Vec<Option<usize>>>>,
    global_best_select_objects: Vec<Object>,
    global_best_cost: usize,
    current_solution: Vec<Vec<Vec<Option<usize>>>>,
    current_selected_objects: Vec<Object>,
}

impl Solve {
    fn new(limit: Limit, objects: Vec<Object>) -> Self {
        let l = limit.x;
        let w = limit.y;
        let h = limit.z;

        let n = objects.len();

        // 1) 初始化 DFS 的物体顺序：0..n-1
        let mut order: Vec<usize> = (0..n).collect();

        // 2) 按价值从大到小排序（价值高的优先）
        order.sort_by(|&i, &j| objects[j].value.cmp(&objects[i].value));

        // 3) 预计算后缀最大剩余价值，用于剪枝
        // suffix_values[pos] = sum_{k=pos..n-1} value(order[k])
        let mut suffix_values = vec![0usize; n + 1];
        for pos in (0..n).rev() {
            let idx = order[pos];
            suffix_values[pos] = suffix_values[pos + 1] + objects[idx].value;
        }

        let empty = vec![vec![vec![None; h]; w]; l];
        Solve {
            limit,
            objects,
            order,
            suffix_values,
            current_solution: empty.clone(),
            global_best_solution: empty,
            global_best_select_objects: vec![],
            global_best_cost: 0,
            current_selected_objects: vec![],
        }
    }

    /// 回溯搜索：按照 order_pos 对应的顺序尝试装物体
    fn dfs(&mut self, order_pos: usize, prev_cost: usize) {
        // 剪枝：即使把后面所有物体都装上，理论最大也比不过当前全局最优
        if prev_cost + self.suffix_values[order_pos] <= self.global_best_cost {
            return;
        }

        // 触底：所有物体都处理完了
        if order_pos == self.order.len() {
            if prev_cost > self.global_best_cost {
                self.global_best_cost = prev_cost;
                self.global_best_solution = self.current_solution.clone();
                self.global_best_select_objects = self.current_selected_objects.clone();
            }
            return;
        }

        // 当前要决策的物体索引（原始 objects 的下标）
        let object_index = self.order[order_pos];

        let (obj_x, obj_y, obj_z, obj_val) = {
            let obj = &self.objects[object_index];
            (obj.x, obj.y, obj.z, obj.value)
        };

        // 分支一：不装当前物体
        self.dfs(order_pos + 1, prev_cost);

        // 尺寸超过限制，直接放弃“尝试装”的分支
        if obj_x > self.limit.x || obj_y > self.limit.y || obj_z > self.limit.z {
            return;
        }

        // 分支二：尝试装当前物体
        let max_x = self.limit.x - obj_x + 1;
        let max_y = self.limit.y - obj_y + 1;
        let max_z = self.limit.z - obj_z + 1;

        for x in 0..max_x {
            for y in 0..max_y {
                for z in 0..max_z {
                    if self.can_place_object(object_index, x, y, z) {
                        let obj_clone = self.objects[object_index].clone();

                        // 选择
                        self.place(object_index, x, y, z);
                        self.current_selected_objects.push(obj_clone);

                        // 递归
                        self.dfs(order_pos + 1, prev_cost + obj_val);

                        // 回溯
                        self.current_selected_objects.pop();
                        self.remove(object_index, x, y, z);
                    }
                }
            }
        }
    }

    /// 检查从 (x, y, z) 开始，第 index 个 object 这块体积是否都为空
    fn can_place_object(&self, object_index: usize, start_x: usize, start_y: usize, start_z: usize) -> bool {
        let obj = &self.objects[object_index];

        // 避免越界
        if start_x + obj.x > self.limit.x
            || start_y + obj.y > self.limit.y
            || start_z + obj.z > self.limit.z
        {
            return false;
        }

        for i in start_x..start_x + obj.x {
            for j in start_y..start_y + obj.y {
                for k in start_z..start_z + obj.z {
                    if self.current_solution[i][j][k].is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// 把第 object_index 个物体放到 (start_x, start_y, start_z)
    fn place(&mut self, object_index: usize, start_x: usize, start_y: usize, start_z: usize) {
        let obj = &self.objects[object_index];
        for i in start_x..start_x + obj.x {
            for j in start_y..start_y + obj.y {
                for k in start_z..start_z + obj.z {
                    self.current_solution[i][j][k] = Some(object_index);
                }
            }
        }
    }

    /// 把第 object_index 个物体从 (start_x, start_y, start_z) 移除
    fn remove(&mut self, object_index: usize, start_x: usize, start_y: usize, start_z: usize) {
        let obj = &self.objects[object_index];
        for i in start_x..start_x + obj.x {
            for j in start_y..start_y + obj.y {
                for k in start_z..start_z + obj.z {
                    self.current_solution[i][j][k] = None;
                }
            }
        }
    }
}

/// 求解并返回：
/// - names: Vec<String>  —— 参与摆放的物体名称数组
/// - grid_zxy: [z][x][y] —— -1 表示空；>=0 表示 names 的索引
/// - selected: Vec<Object> —— 最优解中选中的物体（含字符串 name）
/// - best: usize —— 最优价值
fn solve_to_grid_zxy(
    limit: Limit,
    objects: Vec<Object>,
) -> (Vec<String>, Vec<Vec<Vec<isize>>>, Vec<Object>, usize) {
    let mut solver = Solve::new(limit, objects);
    solver.dfs(0, 0);

    let l = solver.limit.x; // x
    let w = solver.limit.y; // y
    let h = solver.limit.z; // z

    // 1) 统计在解中实际被使用到的物体“原始索引”，按升序（稳定）
    let mut used_indices = BTreeSet::new();
    for x in 0..l {
        for y in 0..w {
            for z in 0..h {
                if let Some(idx) = solver.global_best_solution[x][y][z] {
                    used_indices.insert(idx);
                }
            }
        }
    }

    // 2) 生成名称数组（与 used_indices 的顺序一致）
    let names: Vec<String> = used_indices
        .iter()
        .map(|&idx| solver.objects[idx].name.clone())
        .collect();

    // 3) 建立 “原始物体索引 idx -> names 中的位置 pos（i32）” 的映射
    let mut idx_to_pos: Vec<i32> = vec![-1; solver.objects.len()];
    for (pos, &idx) in used_indices.iter().enumerate() {
        idx_to_pos[idx] = pos as i32;
    }

    // 4) 构建 [z][x][y] 的栅格：空=-1，非空=names 的索引
    let mut grid_zxy = vec![vec![vec![-1isize; w]; l]; h];
    for x in 0..l {
        for y in 0..w {
            for z in 0..h {
                if let Some(idx) = solver.global_best_solution[x][y][z] {
                    let pos = idx_to_pos[idx];
                    grid_zxy[z][x][y] = pos as isize; // -1 或 >=0
                }
            }
        }
    }

    // 5) 选中的物体列表与最优价值
    let selected = solver.global_best_select_objects.clone();
    let best = solver.global_best_cost;

    (names, grid_zxy, selected, best)
}

#[derive(Serialize)]
pub struct SolveOutput {
    pub names: Vec<String>,                // 字符串名称数组
    pub grid_zxy: Vec<Vec<Vec<i32>>>,      // [z][x][y]；空=-1；非空=names 索引
    pub selected: Vec<Object>,             // 最优解中选中的物体
    pub best: usize,                       // 最优价值
}

#[wasm_bindgen]
pub fn solve_to_grid_zxy_js(limit: JsValue, objects: JsValue) -> Result<JsValue, JsValue> {
    // 反序列化前端传入参数（字段名需与 Limit/Object 定义一致）
    let limit: Limit = serde_wasm_bindgen::from_value(limit)?;
    let objects: Vec<Object> = serde_wasm_bindgen::from_value(objects)?;

    // 调用内部求解
    let (names, grid_isize, selected, best) = solve_to_grid_zxy(limit, objects);

    // isize -> i32（WASM/JS 更友好）
    let grid_zxy: Vec<Vec<Vec<i32>>> = grid_isize
        .into_iter()
        .map(|plane| {
            plane
                .into_iter()
                .map(|row| row.into_iter().map(|v| v as i32).collect())
                .collect()
        })
        .collect();

    let out = SolveOutput {
        names,
        grid_zxy,
        selected,
        best,
    };

    Ok(serde_wasm_bindgen::to_value(&out)?)
}
