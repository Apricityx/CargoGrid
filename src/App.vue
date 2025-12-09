<template>
  <div class="app-container">
    <div class="left">
      <v-checkbox
          v-for="(layer, index) in objectCollection"
          :key="index"
          v-model="checkedLayers[index]"
          :label="`Layer ${index}`"
      />
    </div>

    <div class="right">
      <RenderScene :objectCollection="displayedObjectCollection"/>
    </div>
  </div>
</template>

<script setup lang="ts">
import {ref, computed, onMounted} from 'vue'
import RenderScene from './components/RenderScene.vue'
import init, { solve_to_grid_zxy_js } from 'packing_solver'
import wasmUrl from 'packing_solver/index_bg.wasm?url'
type Limit = { length: number; width: number; height: number }
type Obj   = { name: number; length: number; width: number; height: number; value: number }
const objectCollection = ref<number[][][]>([
  [
    [-1, -1, -1],
    [2, 0, 3],
    [0, 0, 0],
    [0, 0, 0]
  ],
  [
    [4, 0, 5],
    [0, 6, 0],
    [7, 0, 8],
    [1, 1, 1]
  ],
  [
    [0, 9, 0],
    [10, 0, 11],
    [0, 0, 0],
    [1, 2, 0]
  ]
])
onMounted(async () => {
  await init(wasmUrl) // 显式传入 URL，避免 ESM-integration
  const limit: Limit = { length: 4, width: 3, height: 3 }
  const objects: Obj[] = [
    { name: 101, length: 2, width: 2, height: 1, value: 6 },
    { name: 202, length: 1, width: 3, height: 1, value: 5 },
    { name: 303, length: 1, width: 2, height: 2, value: 7 },
    { name: 404, length: 2, width: 1, height: 1, value: 3 },
  ]

  // 直接把 JS 对象传进去，得到普通 JS 对象返回
  const result = solve_to_grid_zxy_js(limit, objects) as {
    names: number[]
    grid_zxy: number[][][]  // [z][x][y]，空位=-1
    selected: Obj[]
    best: number
  }

  console.log('names:', result.names)
  console.log('best:', result.best)
  console.log('grid_zxy:', result.grid_zxy)
  // 例如：根据 grid_zxy 渲染 three.js 体素……
})
// objectCollection[layer][row][col]  -> 每个元素是一张 XY 平面，沿着“层”(高度) 叠起来


// 每一层的勾选状态
const checkedLayers = ref<boolean[]>(objectCollection.value.map(() => true))

// 取消勾选的层 -> 变成全 0
const displayedObjectCollection = computed(() =>
    objectCollection.value.map((layer, index) => {
      if (checkedLayers.value[index]) return layer
      return layer.map(row => row.map(() => -1))
    })
)
</script>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
}

.app-container {
  height: 100vh;
  width: 100vw;
  display: grid;
  grid-template-columns: 30% 1fr; /* 左侧定宽*/
}

.left {
  background-color: #e7a4a4;
  padding: 12px;
  box-sizing: border-box;
  overflow-y: auto; /* layer 多了只在左侧滚动 */
}

/* 右侧：交给 Three.js，宽高 100% */
.right {
  position: relative;
  overflow: hidden;
}
</style>
