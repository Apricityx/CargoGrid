<template>
  <div class="app-container">
    <!-- 左侧控制面板 -->
    <div class="left">
      <v-card class="d-flex flex-column" style="height: 100%;" elevation="3">

        <v-card-title
            class="text-subtitle-1 font-weight-bold d-flex align-center justify-space-between"
        >
          <span>控制面板</span>

        </v-card-title>

        <v-divider />

        <v-card-text class="flex-grow-1 overflow-y-auto" style="min-height: 0;">
          <v-expansion-panels style="margin-bottom: 20px">
            <v-expansion-panel
                title="项目背景"
            >
              <v-expansion-panel-text>
                <div style="text-align: left">
                  <p>
                    在现代国际物流与海运出口贸易中，海运运费高昂且舱位资源紧缺。对于出口企业而言，如何在有限的集装箱空间内装载总经济价值最高的货物，是提高利润率的关键环节。
                  </p>
                  <br/>
                  <p>
                    本软件提供了最佳集装箱装载计算与可视化系统，利用回溯算法，帮助企业在满足空间和重量限制的前提下，最大化集装箱内货物的总价值。
                  </p>
                </div>
              </v-expansion-panel-text>
            </v-expansion-panel>
            <v-expansion-panel
                title="数学模型"
            >
              <v-expansion-panel-text>
                <div style="text-align: left">
                  <p>
                    将此问题简化并为一个数学模型，给定一个长方体集装箱，其长、宽、高确定。给定n种类型的待装货物，每种货物具有固定的长宽高以及特定的经济价值。
                    系统需要确定一个装载方案，满足以下约束条件：
                  </p>
                  <v-list lines="two">
                    <v-list-item
                        title="几何约束"
                        subtitle="所有被选中的货物必须完全位于集装箱内部，且任意两个货物在空间上互不重叠。"
                    ></v-list-item>
                    <v-list-item
                        title="方向约束"
                        subtitle="货物必须平行于集装箱的坐标轴放置，不能旋转。"
                    ></v-list-item>
                    <v-list-item
                        title="目标函数"
                        subtitle="在满足上述条件的前提下，最大化装入集装箱内货物的总价值。"
                    ></v-list-item>
                  </v-list>
                </div>
              </v-expansion-panel-text>
            </v-expansion-panel>
          </v-expansion-panels>
          <div class="section-title mb-2">最优解</div>
          <div class="d-flex align-center justify-center gap-2 w-100%">
            <v-chip  size="small" color="primary" variant="flat">
              {{ objectSolution.length }} 层
            </v-chip>
            <v-chip size="small" color="success" variant="flat" v-if="best > 0">
              最优价值: {{ best }}
            </v-chip>

            <v-chip size="small" color="primary" variant="flat" v-if="best > 0">
              货物数量: {{ selected.length }}
            </v-chip>
          </div>
          <div class="section-title mb-2">空间限制</div>


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
            名称不能为空
          </v-alert>

          <v-alert v-if="duplicateName" type="error" variant="tonal" class="mt-2" density="compact">
            名称已存在，请更换一个。
          </v-alert>

          <v-alert v-if="exceedsLimit" type="warning" variant="tonal" class="mt-2" density="compact">
            货物尺寸过大，超过当前空间限制
          </v-alert>

          <v-alert v-if="objects.length >= 10" type="error" variant="tonal" class="mt-2" density="compact">
            物体数量已达上限，无法添加更多物体
          </v-alert>

          <div class="d-flex gap-2 mt-3">
            <v-btn color="primary" size="small" :disabled="!canAdd" @click="addObject">
              添加物体
            </v-btn>
            <v-btn size="small" variant="text" color="secondary" @click="resetNew">
              重置输入
            </v-btn>
          </div>
<!--          货物列表-->
          <div class="section-title mb-2">
            货物列表
          </div>

          <v-table density="comfortable" class="objects-table">
            <thead>
            <tr>
              <th style="width: 160px; text-align: center">商品名</th>
              <th>X</th>
              <th>Y</th>
              <th>Z</th>
              <th style="width: 90px; text-align: center">价值</th>
              <th style="width: 90px; text-align: center">选择</th>
              <th style="width: 90px; text-align: center">操作</th>
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
                <v-chip v-else color="error"
                        size="x-small" variant="flat">未选中</v-chip>
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
          <v-divider/>
<!--          可见图层-->
          <div class="section-title mb-2">可见图层</div>
          <v-sheet class="layers-sheet" rounded="lg" variant="outlined">
            <v-checkbox v-for="(layer, index) in objectSolution" :key="index"
                        v-model="checkedLayers[index]" :label="`Layer z = ${index}`"
                        density="compact" hide-details />
          </v-sheet>
        </v-card-text>

        <v-divider/>
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

const objectSolution = ref<number[][][]>([[[1]]])
const limit: Ref<Limit> = ref({ x: 4, y: 4, z: 2 })

const objects: Ref<Obj[]> = ref([
  { name: '羊肉', x: 2, y: 2, z: 1, value: 6 },
  { name: '牛奶', x: 1, y: 2, z: 1, value: 5 },
  { name: '饼干', x: 1, y: 2, z: 2, value: 7 },
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
        Number.isFinite(newObj.value.value) && newObj.value.value >= 0 &&
        objects.value.length < 10
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
  objectSolution.value = result.grid_zxy
  selected.value = result.selected || []
  best.value = result.best || 0
  names.value = result.names || []
}

const randomNameList = [
  '咸鱼','鸡腿','鸭脖','火腿肠','豆腐干','花生米','榨菜','月饼','糖葫芦',
  '包子','馒头','烧饼','麻花','粽子','牛肉干','肉松','笋干','海带结','酱牛肉',
  '腊肠','腊肉','肉粽','酸菜','泡菜','豆瓣酱','花椒','辣椒面','大米','面粉',
  '食用油','白砂糖','盐包','酱油','醋瓶','巧克力','糖果','矿泉水',
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
  recompute()
})

watch(limit, recompute, { deep: true })
watch(objects, recompute, { deep: true })
watch(objectSolution, (val) => {
  checkedLayers.value = val.map((_, idx) => checkedLayers.value[idx] ?? true)
}, { immediate: true })

const displayedObjectCollection = computed(() =>
    objectSolution.value.map((layer, index) => {
      if (checkedLayers.value[index]) return layer
      return layer.map(row => row.map(() => -1))
    })
)

const resetLayers = () => {
  checkedLayers.value = objectSolution.value.map(() => true)
}
</script>

<style>
:root {
  /* 物体列表滚动区的固定高度 */
  --objects-height: 140px;
}


.app-container {
  height: 100%;
  width: 100%;
  display: grid;
  grid-template-columns: 500px 1fr; /* 左侧固定宽度 */
}

.left {
  padding: 16px;
  box-sizing: border-box;
  overflow-y: auto;
}

.section-title {
  margin-top: 10px;
  font-size: 0.85rem;
  font-weight: 600;
  opacity: 0.7;
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
