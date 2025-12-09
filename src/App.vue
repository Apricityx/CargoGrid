<template>
  <div class="app-container">
    <!-- 左侧控制面板 -->
    <div class="left">
      <v-card class="left-card" elevation="3">
        <v-card-title
            class="text-subtitle-1 font-weight-bold d-flex align-center justify-space-between"
        >
          <span>控制面板</span>
          <v-chip size="small" color="primary" variant="flat">
            {{ objectCollection.length }} 层
          </v-chip>
        </v-card-title>

        <v-divider />
        <!-- Limit 设置，仅作为展示/调整面板 -->
        <v-card-text class="pt-0">
          <div class="section-title mb-2">
            空间限制 Limit
          </div>

          <v-row dense>
            <v-col cols="4">
              <v-text-field
                  v-model.number="limit.x"
                  label="X"
                  type="number"
                  min="1"
                  variant="outlined"
                  density="compact"
                  hide-details
              />
            </v-col>
            <v-col cols="4">
              <v-text-field
                  v-model.number="limit.y"
                  label="Y"
                  type="number"
                  min="1"
                  variant="outlined"
                  density="compact"
                  hide-details
              />
            </v-col>
            <v-col cols="4">
              <v-text-field
                  v-model.number="limit.z"
                  label="Z"
                  type="number"
                  min="1"
                  variant="outlined"
                  density="compact"
                  hide-details
              />
            </v-col>
          </v-row>
        </v-card-text>

        <v-divider />

        <!-- 变更图层 -->
        <v-card-text class="pt-4 pb-2">
          <div class="section-title mb-2">可见图层</div>

          <v-sheet class="layers-sheet" rounded="lg" variant="outlined">
            <v-checkbox
                v-for="(layer, index) in objectCollection"
                :key="index"
                v-model="checkedLayers[index]"
                :label="`Layer z = ${index}`"
                density="compact"
                hide-details
            />
          </v-sheet>
        </v-card-text>



        <v-spacer />

        <v-divider />

        <v-card-actions class="justify-end">
          <v-btn size="small" variant="text" @click="resetLayers">
            全部显示
          </v-btn>
        </v-card-actions>
      </v-card>
    </div>

    <!-- 右侧 Three.js 画布 -->
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
  [[1]],
])

// Limit
const limit: Ref<Limit> = ref({ x: 5, y: 5, z: 5 })

const objects: Ref<Obj[]> = ref([
  { name: 1, x: 2, y: 2, z: 1, value: 6 },
  { name: 2, x: 1, y: 3, z: 1, value: 5 },
  { name: 3, x: 1, y: 2, z: 2, value: 7 },
  { name: 4, x: 2, y: 2, z: 3, value: 10000 },
  { name: 6, x: 3, y: 2, z: 2, value: 10000 },
])

// wasm 是否已经初始化完
const solverReady = ref(false)

// 每一层 z 的勾选状态
const checkedLayers = ref<boolean[]>([])

/** 校验 limit 是否合理 */
const isValidLimit = (l: Limit) =>
    Number.isFinite(l.x) && l.x > 0 &&
    Number.isFinite(l.y) && l.y > 0 &&
    Number.isFinite(l.z) && l.z > 0

/** 根据当前 limit 和 objects 重新计算 objectCollection */
const recompute = () => {
  if (!solverReady.value) return
  if (!isValidLimit(limit.value)) return

  const result = solve_to_grid_zxy_js(limit.value, objects.value) as {
    names: number[]
    grid_zxy: number[][][]
    selected: Obj[]
    best: number
  }

  console.log('names:', result.names)
  console.log('best:', result.best)
  console.log('grid_zxy:', result.grid_zxy)

  objectCollection.value = result.grid_zxy
}

onMounted(async () => {
  await init(wasmUrl)
  solverReady.value = true

  // 初始化一次 limit（会触发下面的 watch(limit) -> recompute）
  limit.value = { x: 4, y: 4, z: 3 }
})

// 当 limit(x/y/z 任一) 变化时，重新求解
watch(
    limit,
    () => {
      recompute()
    },
    { deep: true }
)

// 层数变 checkedLayers 跟着调整
watch(
    objectCollection,
    (val) => {
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

// 一键全显
const resetLayers = () => {
  checkedLayers.value = objectCollection.value.map(() => true)
}
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
  grid-template-columns: 320px 1fr; /* 左侧固定宽度 */
}

.left {
  padding: 16px;
  box-sizing: border-box;
  overflow-y: auto;
}

.left-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.section-title {
  font-size: 0.85rem;
  font-weight: 600;
  opacity: 0.7;
}

.layers-sheet {
  max-height: 260px;
  overflow-y: auto;
  padding: 8px 12px;
}

/* 右侧：交给 Three.js，宽高 100% */
.right {
  position: relative;
  overflow: hidden;
}
</style>
