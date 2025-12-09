<template>
  <div class="app-container">
    <div class="left">
      <v-checkbox
          v-for="(layer, index) in objectCollection"
          :key="index"
          v-model="checkedLayers[index]"
          :label="`Layer z = ${index}`"
      />
    </div>

    <div class="right">
      <RenderScene :objectCollection="displayedObjectCollection" :limit="limit" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, Ref, watch } from 'vue'
import RenderScene from './components/RenderScene.vue'
import init, { solve_to_grid_zxy_js } from 'packing_solver'
import wasmUrl from 'packing_solver/index_bg.wasm?url'

// Rust: struct Limit { x, y, z }
export type Limit = { x: number; y: number; z: number }

// Rust: struct Object { name, x, y, z, value }
type Obj = { name: number; x: number; y: number; z: number; value: number }

// Rust 返回的 grid_zxy: [z][x][y]，空位=-1
const objectCollection = ref<number[][][]>([
  // 初始随便给个 1 层 1x1 占位，后面会被 wasm 结果覆盖
  [[1]],
])

// 初始化Limit
const limit: Ref<Limit> = ref({ x: 5, y: 5, z: 5 })

const objects: Ref<Obj[]> = ref([
  { name: 1, x: 2, y: 2, z: 1, value: 6 },
  { name: 2, x: 1, y: 3, z: 1, value: 5 },
  { name: 3, x: 1, y: 2, z: 2, value: 7 },
  { name: 4, x: 2, y: 2, z: 3, value: 10000 },
  { name: 6, x: 3, y: 2, z: 2, value: 10000 },
])

onMounted(async () => {
  await init(wasmUrl)

  // Rust Limit
  limit.value = { x: 4, y: 4, z: 3 }

  const result = solve_to_grid_zxy_js(limit.value, objects.value) as {
    names: number[]
    grid_zxy: number[][][] // [z][x][y]，空位=-1
    selected: Obj[]
    best: number
  }

  console.log('names:', result.names)
  console.log('best:', result.best)
  console.log('grid_zxy:', result.grid_zxy)

  objectCollection.value = result.grid_zxy
})

// 每一层 z 的勾选状态
const checkedLayers = ref<boolean[]>([])

// 层数变 checkedLayers 跟着调整
watch(
    objectCollection,
    (val) => {
      // 保留已有勾选状态，不够的补 true
      checkedLayers.value = val.map((_, idx) => checkedLayers.value[idx] ?? true)
    },
    { immediate: true }
)

// 取消勾选的层 该层整层置为 -1 表示不渲染
const displayedObjectCollection = computed(() =>
    objectCollection.value.map((layer, index) => {
      if (checkedLayers.value[index]) return layer
      return layer.map(row => row.map(() => -1))
    })
)
</script>

<style>
html,
body,
#app {
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
