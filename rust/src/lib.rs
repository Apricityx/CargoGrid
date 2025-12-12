use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    /// 物体在 x 轴方向的长度
    pub x: usize,
    /// 物体在 y 轴方向的长度
    pub y: usize,
    /// 物体在 z 轴方向的长度
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

    // 只包含“尺寸可行（<= limit）”的物体索引，并按价值从大到小排序
    order: Vec<usize>,

    // 后缀价值和（只针对 order 中的可行物体）
    suffix_values: Vec<usize>,

    // 一维栅格：-1 表示空；>=0 表示 objects 的下标
    current_solution: Vec<i32>,
    global_best_solution: Vec<i32>,

    // 当前/最优选中物体（存索引，避免频繁 clone String）
    current_selected_indices: Vec<usize>,
    global_best_selected_indices: Vec<usize>,

    global_best_cost: usize,

    // 体积与占用统计（用于体积剪枝）
    object_volumes: Vec<usize>,
    used_volume: usize,
    capacity: usize,

    dim_y: usize,
    dim_z: usize,
}

impl Solve {
    fn new(limit: Limit, objects: Vec<Object>) -> Self {
        let dim_x = limit.x;
        let dim_y = limit.y;
        let dim_z = limit.z;

        let n = objects.len();

        let object_volumes: Vec<usize> = objects
            .iter()
            .map(|o| o.x.saturating_mul(o.y).saturating_mul(o.z))
            .collect();

        // 优化 1 只让“尺寸可行”的物体进入 DFS 顺序与 suffix
        let mut order: Vec<usize> = (0..n)
            .filter(|&i| {
                let o = &objects[i];
                o.x <= dim_x && o.y <= dim_y && o.z <= dim_z
            })
            .collect();

        order.sort_by(|&i, &j| objects[j].value.cmp(&objects[i].value));

        let m = order.len();
        let mut suffix_values = vec![0usize; m + 1];
        for pos in (0..m).rev() {
            let idx = order[pos];
            suffix_values[pos] = suffix_values[pos + 1] + objects[idx].value;
        }

        let cells = dim_x * dim_y * dim_z;
        let empty = vec![-1i32; cells];

        Self {
            limit,
            objects,
            order,
            suffix_values,
            current_solution: empty.clone(),
            global_best_solution: empty,
            current_selected_indices: vec![],
            global_best_selected_indices: vec![],
            global_best_cost: 0,
            object_volumes,
            used_volume: 0,
            capacity: cells,
            dim_y,
            dim_z,
        }
    }

    #[inline]
    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        (x * self.dim_y + y) * self.dim_z + z
    }

    fn dfs(&mut self, order_pos: usize, prev_cost: usize) {
        // 剪枝 上界 只包含尺寸可行物体的 suffix
        if prev_cost + self.suffix_values[order_pos] <= self.global_best_cost {
            return;
        }

        // 若已装满，直接尝试更新 best
        if self.used_volume == self.capacity {
            if prev_cost > self.global_best_cost {
                self.global_best_cost = prev_cost;
                self.global_best_solution = self.current_solution.clone();
                self.global_best_selected_indices = self.current_selected_indices.clone();
            }
            return;
        }

        // 触底：所有可行物体都处理完了
        if order_pos == self.order.len() {
            if prev_cost > self.global_best_cost {
                self.global_best_cost = prev_cost;
                self.global_best_solution = self.current_solution.clone();
                self.global_best_selected_indices = self.current_selected_indices.clone();
            }
            return;
        }

        let object_index = self.order[order_pos];
        let obj = &self.objects[object_index];
        let obj_val = obj.value;
        let obj_vol = self.object_volumes[object_index];
        let remaining_free = self.capacity - self.used_volume;

        // 优化2 先尝试“放入”，更快拿到大 best 触发剪枝
        if obj_vol <= remaining_free {
            let max_x = self.limit.x - obj.x + 1;
            let max_y = self.limit.y - obj.y + 1;
            let max_z = self.limit.z - obj.z + 1;

            'outer: for x in 0..max_x {
                for y in 0..max_y {
                    for z in 0..max_z {
                        if self.can_place_object(object_index, x, y, z) {
                            self.place(object_index, x, y, z);
                            self.current_selected_indices.push(object_index);

                            self.dfs(order_pos + 1, prev_cost + obj_val);

                            self.current_selected_indices.pop();
                            self.remove(object_index, x, y, z);

                        }
                    }
                }
            }
        }

        // 分支：不放当前物体
        self.dfs(order_pos + 1, prev_cost);
    }

    fn can_place_object(&self, object_index: usize, start_x: usize, start_y: usize, start_z: usize) -> bool {
        let obj = &self.objects[object_index];

        if start_x + obj.x > self.limit.x || start_y + obj.y > self.limit.y || start_z + obj.z > self.limit.z {
            return false;
        }

        for x in start_x..start_x + obj.x {
            for y in start_y..start_y + obj.y {
                let base = self.idx(x, y, start_z);
                // 连续 z 段检查
                for dz in 0..obj.z {
                    if self.current_solution[base + dz] != -1 {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn place(&mut self, object_index: usize, start_x: usize, start_y: usize, start_z: usize) {
        let obj = &self.objects[object_index];
        for x in start_x..start_x + obj.x {
            for y in start_y..start_y + obj.y {
                let base = self.idx(x, y, start_z);
                for dz in 0..obj.z {
                    self.current_solution[base + dz] = object_index as i32;
                }
            }
        }
        self.used_volume += self.object_volumes[object_index];
    }

    fn remove(&mut self, object_index: usize, start_x: usize, start_y: usize, start_z: usize) {
        let obj = &self.objects[object_index];
        for x in start_x..start_x + obj.x {
            for y in start_y..start_y + obj.y {
                let base = self.idx(x, y, start_z);
                for dz in 0..obj.z {
                    self.current_solution[base + dz] = -1;
                }
            }
        }
        self.used_volume -= self.object_volumes[object_index];
    }
}

/// 求解并返回：
/// - names: Vec<String>  —— 参与摆放的物体名称数组
/// - grid_zxy: [z][x][y] —— -1 表示空；>=0 表示 names 的索引
/// - selected: Vec<Object> —— 最优解中选中的物体（含字符串 name）
/// - best: usize —— 最优价值
fn solve_to_grid_zxy(limit: Limit, objects: Vec<Object>) -> (Vec<String>, Vec<Vec<Vec<isize>>>, Vec<Object>, usize) {
    let mut solver = Solve::new(limit, objects);
    solver.dfs(0, 0);

    let l = solver.limit.x; // x
    let w = solver.limit.y; // y
    let h = solver.limit.z; // z

    let mut used_indices = BTreeSet::new();
    for x in 0..l {
        for y in 0..w {
            for z in 0..h {
                let v = solver.global_best_solution[solver.idx(x, y, z)];
                if v >= 0 {
                    used_indices.insert(v as usize);
                }
            }
        }
    }

    let names: Vec<String> = used_indices
        .iter()
        .map(|&idx| solver.objects[idx].name.clone())
        .collect();

    let mut idx_to_pos: Vec<i32> = vec![-1; solver.objects.len()];
    for (pos, &idx) in used_indices.iter().enumerate() {
        idx_to_pos[idx] = pos as i32;
    }

    let mut grid_zxy = vec![vec![vec![-1isize; w]; l]; h];
    for x in 0..l {
        for y in 0..w {
            for z in 0..h {
                let v = solver.global_best_solution[solver.idx(x, y, z)];
                if v >= 0 {
                    let idx = v as usize;
                    let pos = idx_to_pos[idx];
                    grid_zxy[z][x][y] = pos as isize;
                }
            }
        }
    }

    // 5) 选中的物体列表与最优价值
    let selected: Vec<Object> = solver
        .global_best_selected_indices
        .iter()
        .map(|&i| solver.objects[i].clone())
        .collect();

    let best = solver.global_best_cost;
    (names, grid_zxy, selected, best)
}

#[derive(Serialize)]
pub struct SolveOutput {
    pub names: Vec<String>,           // 字符串名称数组
    pub grid_zxy: Vec<Vec<Vec<i32>>>, // [z][x][y]；空=-1；非空=names 索引
    pub selected: Vec<Object>,        // 最优解中选中的物体
    pub best: usize,                  // 最优价值
}

#[wasm_bindgen]
pub fn solve_to_grid_zxy_js(limit: JsValue, objects: JsValue) -> Result<JsValue, JsValue> {
    let limit: Limit = serde_wasm_bindgen::from_value(limit)?;
    let objects: Vec<Object> = serde_wasm_bindgen::from_value(objects)?;

    let (names, grid_isize, selected, best) = solve_to_grid_zxy(limit, objects);

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
