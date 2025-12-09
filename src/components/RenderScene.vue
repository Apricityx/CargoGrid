<template>
  <div ref="containerRef" class="three-container"></div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'

// objectCollection[layer][row][col]
// layer: 高度方向的一层
// row:   平面里的 y 索引
// col:   平面里的 x 索引
const props = defineProps<{
  objectCollection: number[][][]
}>()

const containerRef = ref<HTMLDivElement | null>(null)

let renderer: THREE.WebGLRenderer | null = null
let scene: THREE.Scene | null = null
let camera: THREE.PerspectiveCamera | null = null
let controls: OrbitControls | null = null
let animationId = 0

let voxelGroup: THREE.Group | null = null
let gridHelper: THREE.Box3Helper | null = null

const boxGeometry = new THREE.BoxGeometry(1, 1, 1)
const materialCache = new Map<number, THREE.MeshStandardMaterial>()

function getMaterialForValue(value: number) {
  let m = materialCache.get(value)
  if (m) return m

  const hue = ((value * 2654435761) >>> 0) / 0xffffffff
  const color = new THREE.Color()
  color.setHSL(hue, 0.6, 0.55)

  m = new THREE.MeshStandardMaterial({ color })
  materialCache.set(value, m)
  return m
}

/**
 * 根据 collection 重建体素
 */
function buildVoxels(collection: number[][][], fitCamera = false) {
  if (!scene) return

  if (voxelGroup) {
    scene.remove(voxelGroup)
    voxelGroup = null
  }
  if (gridHelper) {
    scene.remove(gridHelper)
    gridHelper = null
  }

  const NZ = collection.length
  if (NZ === 0) return

  const NY = collection[0]?.length ?? 0
  const NX = collection[0]?.[0]?.length ?? 0
  if (NY === 0 || NX === 0) return

  voxelGroup = new THREE.Group()

  // 映射：col -> X, layer -> Y(高度), row -> Z
  for (let layer = 0; layer < NZ; layer++) {
    for (let row = 0; row < NY; row++) {
      for (let col = 0; col < NX; col++) {
        const v = collection[layer][row][col]
        if (v <= -1) continue

        const material = getMaterialForValue(v)
        const cube = new THREE.Mesh(boxGeometry, material)
        cube.position.set(col, layer, row)
        voxelGroup.add(cube)
      }
    }
  }

  scene.add(voxelGroup)

  // 边框
  const min = new THREE.Vector3(-0.5, -0.5, -0.5)
  const max = new THREE.Vector3(NX - 0.5, NZ - 0.5, NY - 0.5)
  const gridBox = new THREE.Box3(min, max)
  gridHelper = new THREE.Box3Helper(gridBox, 0x333333)
  scene.add(gridHelper)

  if (fitCamera && camera) {
    const box = new THREE.Box3().setFromObject(voxelGroup)
    const center = box.getCenter(new THREE.Vector3())
    const size = box.getSize(new THREE.Vector3())
    const maxDim = Math.max(size.x, size.y, size.z || 1)

    const fitHeightDistance =
        maxDim / (2 * Math.tan(THREE.MathUtils.degToRad(camera.fov) / 2))
    const distance = fitHeightDistance * 1.3

    camera.position.set(center.x + distance, center.y + distance, center.z + distance)
    camera.lookAt(center)

    if (controls) {
      controls.target.copy(center)
      controls.update()
    }
  }
}

function initScene() {
  const container = containerRef.value
  if (!container) return

  const width = container.clientWidth
  const height = container.clientHeight

  scene = new THREE.Scene()
  scene.background = new THREE.Color(0xdddddd)

  renderer = new THREE.WebGLRenderer({ antialias: true })
  renderer.setPixelRatio(window.devicePixelRatio)
  renderer.setSize(width, height)
  container.appendChild(renderer.domElement)

  camera = new THREE.PerspectiveCamera(60, width / height, 0.1, 1000)
  scene.add(camera)

  scene.add(new THREE.AmbientLight(0xffffff, 0.5))
  const dir = new THREE.DirectionalLight(0xffffff, 0.9)
  dir.position.set(5, 8, 10)
  scene.add(dir)

  buildVoxels(props.objectCollection, true)

  controls = new OrbitControls(camera, renderer.domElement)
  controls.enablePan = false
  controls.autoRotate = true
  controls.autoRotateSpeed = 1

  if (voxelGroup) {
    const box = new THREE.Box3().setFromObject(voxelGroup)
    const center = box.getCenter(new THREE.Vector3())
    controls.target.copy(center)
  }
  controls.update()

  const animate = () => {
    animationId = requestAnimationFrame(animate)
    controls!.update()
    renderer!.render(scene!, camera!)
  }
  animate()
}

function handleResize() {
  const container = containerRef.value
  if (!container || !renderer || !camera) return
  const w = container.clientWidth
  const h = container.clientHeight
  camera.aspect = w / h
  camera.updateProjectionMatrix()
  renderer.setSize(w, h)
}

onMounted(() => {
  initScene()
  window.addEventListener('resize', handleResize)
  handleResize()
})

watch(
    () => props.objectCollection,
    (newVal) => {
      if (!scene) return
      buildVoxels(newVal, false) // 更新体素，不动相机角度
    },
    { deep: true }
)

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize)
  if (animationId) cancelAnimationFrame(animationId)

  if (renderer) {
    renderer.dispose()
    renderer.forceContextLoss?.()
    renderer.domElement.remove()
    renderer = null
  }

  boxGeometry.dispose()
  materialCache.forEach(m => m.dispose())
  materialCache.clear()

  scene = null
  camera = null
  controls = null
  voxelGroup = null
  gridHelper = null
})
</script>

<style scoped>
.three-container {
  width: 100%;
  height: 100%;
  border: 1px solid #ddd;
  overflow: hidden;
}
</style>
