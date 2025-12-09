<template>
  <div ref="containerRef" class="three-container"></div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'
import type { Limit } from '../App.vue'

// objectCollection: [z][x][y]，空位 = -1
const props = defineProps<{
  objectCollection: number[][][]
  limit: Limit      // { x, y, z }
}>()

const containerRef = ref<HTMLDivElement | null>(null)

let renderer: THREE.WebGLRenderer | null = null
let scene: THREE.Scene | null = null
let camera: THREE.PerspectiveCamera | null = null
let controls: OrbitControls | null = null
let animationId = 0

let voxelGroup: THREE.Group | null = null
let gridHelper: THREE.Box3Helper | null = null
let axisGroup: THREE.Group | null = null

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

// 文字 Sprite 标注 X/Y/Z
function createTextSprite(text: string, color = '#000000'): THREE.Sprite {
  const canvas = document.createElement('canvas')
  const size = 256
  canvas.width = size
  canvas.height = size

  const ctx = canvas.getContext('2d')!
  ctx.clearRect(0, 0, size, size)
  ctx.fillStyle = color
  ctx.font = 'bold 80px sans-serif'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillText(text, size / 2, size / 2)

  const texture = new THREE.CanvasTexture(canvas)
  const material = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
  })

  const sprite = new THREE.Sprite(material)
  sprite.scale.set(1.2, 1.2, 1.2)
  return sprite
}

function getLimitCenter(limit: Limit) {
  return new THREE.Vector3(
      limit.x / 2,
      limit.z / 2, // 高度
      limit.y / 2,
  )
}

/**
 * 根据 collection 重建体素
 * collection: [z][x][y]
 */
function buildVoxels(collection: number[][][], limit: Limit, fitCamera = false) {
  if (!scene) return

  if (voxelGroup) {
    scene.remove(voxelGroup)
    voxelGroup = null
  }
  if (gridHelper) {
    scene.remove(gridHelper)
    gridHelper = null
  }
  if (axisGroup) {
    scene.remove(axisGroup)
    axisGroup = null
  }

  const NZ = collection.length      // z 高度层数
  if (NZ === 0) return

  const NX = collection[0]?.length ?? 0      // x
  const NY = collection[0]?.[0]?.length ?? 0 // y
  if (NX === 0 || NY === 0) return

  voxelGroup = new THREE.Group()

  // 映射：
  //   xIndex -> Three X
  //   zIndex -> Three Y
  //   yIndex -> Three Z
  for (let zIndex = 0; zIndex < NZ; zIndex++) {
    for (let xIndex = 0; xIndex < NX; xIndex++) {
      for (let yIndex = 0; yIndex < NY; yIndex++) {
        const v = collection[zIndex][xIndex][yIndex]
        if (v <= -1) continue

        const material = getMaterialForValue(v)
        const cube = new THREE.Mesh(boxGeometry, material)
        // 放在每个格子的中心
        cube.position.set(xIndex + 0.5, zIndex + 0.5, yIndex + 0.5)
        voxelGroup.add(cube)
      }
    }
  }

  scene.add(voxelGroup)

  // ====== Limit 边框 Box3（[0, x] × [0, z] × [0, y]）======
  const min = new THREE.Vector3(0, 0, 0)
  const max = new THREE.Vector3(
      limit.x,
      limit.z,
      limit.y
  )
  const gridBox = new THREE.Box3(min, max)
  gridHelper = new THREE.Box3Helper(gridBox, 0x333333)
  scene.add(gridHelper)

  /**
   * 画坐标轴
   * 值得注意的是，Three.js中的y代表高度
   */
  axisGroup = new THREE.Group()
  // 坐标轴原点
  const origin = new THREE.Vector3(-0.01, -0.01, -0.01)
  const x_length = limit.x + 2
  const y_length = limit.y + 2
  const z_length = limit.z + 2

  // 坐标轴箭头
  const xArrow = new THREE.ArrowHelper(
      new THREE.Vector3(1, 0, 0),
      origin,
      x_length,
      0xff0000
  )
  const yArrow = new THREE.ArrowHelper(
      new THREE.Vector3(0, 1, 0),
      origin,
      z_length,
      0x0000ff
  )
  const zArrow = new THREE.ArrowHelper(
      new THREE.Vector3(0, 0, 1),
      origin,
      y_length,
      0x00ff00
  )

  // Label
  const xLabel = createTextSprite('X', '#ff0000')
  xLabel.position.set(x_length + 0.4, 0, 0)
  const yLabel = createTextSprite('Y', '#00ff00')
  yLabel.position.set(0, 0, y_length + 0.4)
  const zLabel = createTextSprite('Z', '#0000ff')
  zLabel.position.set(0, z_length + 0.4, 0)

  axisGroup.add(xArrow)
  axisGroup.add(yArrow)
  axisGroup.add(zArrow)
  axisGroup.add(xLabel)
  axisGroup.add(yLabel)
  axisGroup.add(zLabel)

  scene.add(axisGroup)

  /**
   * 相机用 Limit 框的中心
   */
  if (fitCamera && camera) {
    const center = getLimitCenter(limit)
    const size = gridBox.getSize(new THREE.Vector3())
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

  buildVoxels(props.objectCollection, props.limit, true)

  controls = new OrbitControls(camera, renderer.domElement)
  controls.enablePan = false
  controls.autoRotate = true
  controls.autoRotateSpeed = 1

  const center = getLimitCenter(props.limit)
  controls.target.copy(center)
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
      // 只更新体素 不改相机旋转中心
      buildVoxels(newVal, props.limit, false)
    },
    { deep: true }
)

watch(
    () => props.limit,
    (newVal) => {
      if (!scene) return
      // Limit 改变 重新构建盒子 重新把中心调到新 Limit 框中心
      buildVoxels(props.objectCollection, newVal, false)
      if (camera && controls) {
        const center = getLimitCenter(newVal)
        camera.lookAt(center)
        controls.target.copy(center)
        controls.update()
      }
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
  axisGroup = null
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
