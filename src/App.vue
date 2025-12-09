<template>
  <div class="app-container">
    <!-- 左侧控制面板 -->
    <div class="left">
      <v-card class="left-card" elevation="3">
        <v-card-title
            class="text-subtitle-1 font-weight-bold d-flex align-center justify-space-between"
        >
          <span>控制面板</span>
          <div class="d-flex align-center gap-2">
            <v-chip size="small" color="primary" variant="flat">
              {{ objectCollection.length }} 层
            </v-chip>
            <v-chip size="small" color="success" variant="flat" v-if="best > 0">
              Best: {{ best }}
            </v-chip>
          </div>
        </v-card-title>

        <v-divider />

        <!-- Limit 设置 -->
        <v-card-text class="pt-4">
          <div class="section-title mb-2">空间限制 Limit</div>
          <v-row dense>
            <v-col cols="4">
              <v-text-field v-model.number="limit.x" label="X" type="number" min="1"
                            variant="outlined" density="compact" hide-details />
            </v-col>
            <v-col cols="4">
              <v-text-field v-model.number="limit.y" label="Y" type="number" min="1"
                            variant="outlined" density="compact" hide-details />
            </v-col>
            <v-col cols="4">
              <v-text-field v-model.number="limit.z" label="Z" type="number" min="1"
                            variant="outlined" density="compact" hide-details />
            </v-col>
          </v-row>
        </v-card-text>

        <v-divider />

        <!-- 添加物体 -->
        <v-card-text class="pt-4">
          <div class="section-title mb-2">添加物体</div>

          <v-row dense>
            <v-col cols="12">
              <v-text-field v-model="newObj.name" label="名称" type="text"
                            variant="outlined" density="compact" hide-details />
            </v-col>

            <v-col cols="4">
              <v-text-field v-model.number="newObj.x" label="X" type="number" min="1"
                            variant="outlined" density="compact" hide-details />
            </v-col>
            <v-col cols="4">
              <v-text-field v-model.number="newObj.y" label="Y" type="number" min="1"
                            variant="outlined" density="compact" hide-details />
            </v-col>
            <v-col cols="4">
              <v-text-field v-model.number="newObj.z" label="Z" type="number" min="1"
                            variant="outlined" density="compact" hide-details />
            </v-col>

            <v-col cols="12">
              <v-text-field v-model.number="newObj.value" label="Value" type="number" min="0"
                            variant="outlined" density="compact" hide-details />
            </v-col>
          </v-row>

          <v-alert v-if="nameEmpty" type="error" variant="tonal" class="mt-2" density="compact">
            名称不能为空。
          </v-alert>

          <v-alert v-if="duplicateName" type="error" variant="tonal" class="mt-2" density="compact">
            名称已存在，请更换一个。
          </v-alert>

          <v-alert v-if="exceedsLimit" type="warning" variant="tonal" class="mt-2" density="compact">
            物体尺寸超过 Limit，求解时将自动跳过（不会被放置）。
          </v-alert>

          <div class="d-flex gap-2 mt-3">
            <v-btn color="primary" size="small" :disabled="!canAdd" @click="addObject">
              添加物体
            </v-btn>
            <v-btn size="small" variant="text" color="secondary" @click="resetNew">
              重置输入
            </v-btn>
          </div>
        </v-card-text>

        <v-divider />

        <!-- 物体列表（固定高度可滚动窗口） -->
        <v-card-text class="pt-4">
          <div class="section-title mb-2">
            物体列表
          </div>

          <!-- 独立滚动容器：高度用 CSS 变量控制 -->
          <div class="objects-scroller">
            <v-table density="comfortable" class="objects-table">
              <thead>
              <tr>
                <th style="width: 160px;">name</th>
                <th>X</th>
                <th>Y</th>
                <th>Z</th>
                <th>value</th>
                <th style="width: 90px;">状态</th>
                <th style="width: 70px;">操作</th>
              </tr>
              </thead>
              <tbody>
              <tr v-for="(o, idx) in objects" :key="o.name"
                  :class="{ 'row-selected': selectedNameSet.has(o.name) }">
                <td class="mono">{{ o.name }}</td>
                <td>{{ o.x }}</td>
                <td>{{ o.y }}</td>
                <td>{{ o.z }}</td>
                <td>{{ o.value }}</td>
                <td>
                  <v-chip v-if="selectedNameSet.has(o.name)" color="success"
                          size="x-small" variant="flat">选中</v-chip>
                  <span v-else class="text-disabled">—</span>
                </td>
                <td>
                  <v-btn size="x-small" color="error" variant="text"
                         @click="removeObject(idx)">删除</v-btn>
                </td>
              </tr>
              <tr v-if="objects.length === 0">
                <td colspan="7" class="text-center text-disabled">暂无物体</td>
              </tr>
              </tbody>
            </v-table>
          </div>
        </v-card-text>

        <v-divider />

        <!-- 可见图层 -->
        <v-card-text class="pt-4 pb-2">
          <div class="section-title mb-2">可见图层</div>
          <v-sheet class="layers-sheet" rounded="lg" variant="outlined">
            <v-checkbox v-for="(layer, index) in objectCollection" :key="index"
                        v-model="checkedLayers[index]" :label="`Layer z = ${index}`"
                        density="compact" hide-details />
          </v-sheet>
        </v-card-text>

        <v-spacer />

        <v-divider />

<!--        <v-card-actions class="justify-end">-->
<!--          <v-btn size="small" variant="text" @click="resetLayers">-->
<!--            保存-->
<!--          </v-btn>-->
<!--        </v-card-actions>-->
      </v-card>
    </div>

    <!-- 右侧 Three.js 画布 -->
    <div class="right">
      <RenderScene
          :objectCollection="displayedObjectCollection"
          :limit="limit"
          :names="names"
          :valuesMap="valuesMap"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, Ref, watch } from 'vue'
import RenderScene from './components/RenderScene.vue'
import init, { solve_to_grid_zxy_js } from 'packing_solver'
import wasmUrl from 'packing_solver/index_bg.wasm?url'

export type Limit = { x: number; y: number; z: number }
type Obj = { name: string; x: number; y: number; z: number; value: number }

type SolveOut = {
  names: string[]
  grid_zxy: number[][][]
  selected: Obj[]
  best: number
}

const objectCollection = ref<number[][][]>([[[1]]])
const limit: Ref<Limit> = ref({ x: 5, y: 5, z: 5 })

const objects: Ref<Obj[]> = ref([
  { name: 'A', x: 2, y: 2, z: 1, value: 6 },
  { name: 'B', x: 1, y: 3, z: 1, value: 5 },
  { name: 'C', x: 1, y: 2, z: 2, value: 7 },
])

const newObj = ref<Obj>({ name: '方便面', x: 1, y: 1, z: 1, value: 1 })

const solverReady = ref(false)
const selected = ref<Obj[]>([])
const best = ref(0)
const selectedNameSet = computed(() => new Set(selected.value.map(o => o.name)))

const names = ref<string[]>([])
const valuesMap = computed<Record<string, number>>(
    () => Object.fromEntries(objects.value.map(o => [o.name, o.value]))
)

const nameTrimmed = computed(() => (newObj.value.name ?? '').trim())
const nameEmpty = computed(() => nameTrimmed.value.length === 0)
const duplicateName = computed(() => objects.value.some(o => o.name === nameTrimmed.value))
const canAdd = computed(
    () =>
        !nameEmpty.value &&
        !duplicateName.value &&
        Number.isFinite(newObj.value.x) && newObj.value.x > 0 &&
        Number.isFinite(newObj.value.y) && newObj.value.y > 0 &&
        Number.isFinite(newObj.value.z) && newObj.value.z > 0 &&
        Number.isFinite(newObj.value.value) && newObj.value.value >= 0
)
const exceedsLimit = computed(
    () =>
        newObj.value.x > limit.value.x ||
        newObj.value.y > limit.value.y ||
        newObj.value.z > limit.value.z
)

const checkedLayers = ref<boolean[]>([])

const isValidLimit = (l: Limit) =>
    Number.isFinite(l.x) && l.x > 0 &&
    Number.isFinite(l.y) && l.y > 0 &&
    Number.isFinite(l.z) && l.z > 0

const recompute = () => {
  if (!solverReady.value) return
  if (!isValidLimit(limit.value)) return

  const result = solve_to_grid_zxy_js(limit.value, objects.value) as SolveOut
  objectCollection.value = result.grid_zxy
  selected.value = result.selected || []
  best.value = result.best || 0
  names.value = result.names || []
}

const randomNameList = [
  '咸鱼','鸡腿','鸭脖','火腿肠','豆腐干','花生米','榨菜','月饼','糖葫芦',
  '包子','馒头','烧饼','麻花','粽子','牛肉干','肉松','笋干','海带结','酱牛肉',
  '腊肠','腊肉','肉粽','酸菜','泡菜','豆瓣酱','花椒','辣椒面','大米','面粉',
  '食用油','白砂糖','盐包','酱油','醋瓶','牛奶','饼干','巧克力','糖果','矿泉水',
  '绿豆','红枣','燕麦片','玉米粒','茶叶','咖啡豆','纸巾','牙膏','香皂','洗发水'
]
let counter = 0

const addObject = () => {
  if (!canAdd.value) return
  objects.value.push({
    name: nameTrimmed.value,
    x: Math.floor(newObj.value.x),
    y: Math.floor(newObj.value.y),
    z: Math.floor(newObj.value.z),
    value: Math.floor(newObj.value.value),
  })
  recompute()
  if (counter < randomNameList.length - 1) {
    counter++
    newObj.value.name = randomNameList[counter]
  }
}
const removeObject = (idx: number) => {
  objects.value.splice(idx, 1)
  recompute()
}
const resetNew = () => {
  newObj.value = { name: randomNameList[counter], x: 1, y: 1, z: 1, value: 1 }
}

onMounted(async () => {
  await init(wasmUrl)
  solverReady.value = true
  limit.value = { x: 4, y: 4, z: 3 }
})

watch(limit, recompute, { deep: true })
watch(objects, recompute, { deep: true })
watch(objectCollection, (val) => {
  checkedLayers.value = val.map((_, idx) => checkedLayers.value[idx] ?? true)
}, { immediate: true })

const displayedObjectCollection = computed(() =>
    objectCollection.value.map((layer, index) => {
      if (checkedLayers.value[index]) return layer
      return layer.map(row => row.map(() => -1))
    })
)

const resetLayers = () => {
  checkedLayers.value = objectCollection.value.map(() => true)
}
</script>

<style>
:root {
  /* 物体列表滚动区的固定高度 */
  --objects-height: 140px;
}

html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
}

.app-container {
  height: 100vh;
  width: 100vw;
  display: grid;
  grid-template-columns: 460px 1fr; /* 左侧固定宽度 */
}

.left {
  padding: 16px;
  box-sizing: border-box;
  overflow-y: auto; /* 左侧可整体滚动（页面不会跟着滚） */
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

/* 独立滚动窗口：固定高度、表头粘滞 */
.objects-scroller {
  height: var(--objects-height);
  overflow: auto;
  border: 1px solid #eef0f3;
  border-radius: 8px;
  background: #fff;
}

.objects-table thead th {
  position: sticky;
  top: 0;
  background: #fafafa;
  z-index: 1;
  box-shadow: inset 0 -1px 0 #eee;
}

.objects-table td, .objects-table th {
  font-size: 12.5px;
}

.objects-table .row-selected {
  background: #e9f7ef; /* 选中高亮 */
}

.layers-sheet {
  max-height: 260px;
  overflow-y: auto;
  padding: 8px 12px;
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
}

/* 右侧：交给 Three.js，宽高 100% */
.right {
  position: relative;
  overflow: hidden;
}
</style>
