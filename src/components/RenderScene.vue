<template>
  <div ref="containerRef" class="three-container">
    <!-- 悬停提示 -->
    <div
        v-show="tooltip.visible"
        class="hover-tooltip"
        :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
    >
      {{ tooltip.text }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch, reactive } from 'vue'
import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'
import type { Limit } from '../App.vue'

const props = withDefaults(defineProps<{
  objectCollection: number[][][]   // [z][x][y]，空位=-1
  limit: Limit                     // { x, y, z }
  names?: string[]                 // 索引 -> 名称
  valuesMap?: Record<string, number> // 名称 -> 价值
}>(), {
  names: () => [],
  valuesMap: () => ({})
})

const containerRef = ref<HTMLDivElement | null>(null)

let renderer: THREE.WebGLRenderer | null = null
let scene: THREE.Scene | null = null
let camera: THREE.PerspectiveCamera | null = null
let controls: OrbitControls | null = null
let animationId = 0

let voxelGroup: THREE.Group | null = null
let gridHelper: THREE.Box3Helper | null = null
let axisGroup: THREE.Group | null = null
let floorGrid: THREE.LineSegments | null = null  // ★ 新增：自定义矩形棋盘

// 悬停选择
const raycaster = new THREE.Raycaster()
const mouseNdc = new THREE.Vector2()
let hoveredIndex: number | null = null
let indexToMeshes: Map<number, THREE.Mesh[]> = new Map()

const tooltip = reactive({ visible: false, text: '', x: 0, y: 0 })

// 资源
const boxGeometry = new THREE.BoxGeometry(1, 1, 1)
const materialCache = new Map<number, THREE.MeshStandardMaterial>()

function getMaterialForValue(value: number) {
  let m = materialCache.get(value)
  if (m) return m
  const hue = ((value * 2654435761) >>> 0) / 0xffffffff
  const color = new THREE.Color().setHSL(hue, 0.6, 0.55)
  m = new THREE.MeshStandardMaterial({ color, roughness: 0.35, metalness: 0.1 })
  materialCache.set(value, m)
  return m
}

/** 绘制圆角背景的文字 sprite */
function createTextSprite(text: string, options?: {
  color?: string, bg?: string, fontSize?: number, padding?: number, radius?: number, bold?: boolean
}): THREE.Sprite {
  const {
    color = '#111', bg = 'rgba(255,255,255,0.9)',
    fontSize = 42, padding = 10, radius = 10, bold = true
  } = options || {}

  const dpr = Math.min(2, window.devicePixelRatio || 1)
  const estW = (text.length * fontSize * 0.65 + padding * 2) | 0
  const estH = (fontSize + padding * 2) | 0
  const width = Math.max(64, Math.pow(2, Math.ceil(Math.log2(estW * dpr))))
  const height = Math.max(64, Math.pow(2, Math.ceil(Math.log2(estH * dpr))))

  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height
  const ctx = canvas.getContext('2d')!

  const w = width, h = height
  const r = radius * dpr
  ctx.fillStyle = bg
  roundRect(ctx, (w - estW * dpr) / 2, (h - estH * dpr) / 2, estW * dpr, estH * dpr, r)
  ctx.fill()

  ctx.fillStyle = color
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.font = `${bold ? 'bold ' : ''}${fontSize * dpr}px sans-serif`
  ctx.fillText(text, w / 2, h / 2)

  const texture = new THREE.CanvasTexture(canvas)
  texture.anisotropy = 8
  const material = new THREE.SpriteMaterial({ map: texture, transparent: true })
  const sprite = new THREE.Sprite(material)
  const scaleX = Math.max(0.7, Math.min(2.0, estW / 42)) * 0.04
  const scaleY = Math.max(0.7, Math.min(2.0, estH / 42)) * 0.04
  sprite.scale.set(scaleX * 3, scaleY * 3, 1)
  return sprite
}
function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.arcTo(x + w, y, x + w, y + h, r)
  ctx.arcTo(x + w, y + h, x, y + h, r)
  ctx.arcTo(x, y + h, x, y, r)
  ctx.arcTo(x, y, x + w, y, r)
  ctx.closePath()
}

function getLimitCenter(limit: Limit) {
  return new THREE.Vector3(limit.x / 2, limit.z / 2, limit.y / 2)
}

/** —— 美化坐标轴（保持不变） —— */
function addFancyAxes(limit: Limit) {
  axisGroup = new THREE.Group()
  const origin = new THREE.Vector3(-0.01, -0.01, -0.01)

  const lenX = limit.x + 1
  const lenY = limit.z + 1
  const lenZ = limit.y + 1

  const style = { shaftRadius: 0.03, headRadius: 0.09, headLength: 0.35, minorTick: 0.05, majorTick: 0.08 }

  const axisX = buildAxis({ origin, dir: new THREE.Vector3(1,0,0), length: lenX, color: 0xff4d4f,
    tickEvery: 1, majorEvery: 5, label: 'X', labelColor: '#ff4d4f', numberOffset: new THREE.Vector3(0,0.25,0), ...style })
  const axisY = buildAxis({ origin, dir: new THREE.Vector3(0,1,0), length: lenY, color: 0x4169e1,
    tickEvery: 1, majorEvery: 5, label: 'Z', labelColor: '#4169e1', numberOffset: new THREE.Vector3(0.22,0,0), ...style })
  const axisZ = buildAxis({ origin, dir: new THREE.Vector3(0,0,1), length: lenZ, color: 0x2ecc71,
    tickEvery: 1, majorEvery: 5, label: 'Y', labelColor: '#2ecc71', numberOffset: new THREE.Vector3(0,0.25,0), ...style })

  axisGroup.add(axisX, axisY, axisZ)
  scene!.add(axisGroup)
}
function buildAxis(opts: {
  origin: THREE.Vector3, dir: THREE.Vector3, length: number, color: number,
  shaftRadius: number, headRadius: number, headLength: number,
  tickEvery: number, majorEvery: number, label: string, labelColor: string, numberOffset: THREE.Vector3
}) {
  const { origin, dir, length, color, shaftRadius, headRadius, headLength, tickEvery, majorEvery, label, numberOffset } = opts
  const group = new THREE.Group()
  const dirN = dir.clone().normalize()
  const shaftLen = Math.max(0.0001, length - headLength)
  const shaftGeo = new THREE.CylinderGeometry(shaftRadius, shaftRadius, shaftLen, 16, 1)
  const shaftMat = new THREE.MeshStandardMaterial({ color, roughness: 0.2, metalness: 0.6, emissive: new THREE.Color(color).multiplyScalar(0.15) })
  const shaft = new THREE.Mesh(shaftGeo, shaftMat)
  const rotQ = new THREE.Quaternion().setFromUnitVectors(new THREE.Vector3(0,1,0), dirN)
  shaft.quaternion.copy(rotQ)
  shaft.position.copy(origin).add(dirN.clone().multiplyScalar(shaftLen / 2))
  group.add(shaft)
  const headGeo = new THREE.ConeGeometry(headRadius, headLength, 24)
  const headMat = new THREE.MeshStandardMaterial({ color, roughness: 0.15, metalness: 0.8, emissive: new THREE.Color(color).multiplyScalar(0.2) })
  const head = new THREE.Mesh(headGeo, headMat)
  head.quaternion.copy(rotQ)
  head.position.copy(origin).add(dirN.clone().multiplyScalar(shaftLen + headLength / 2))
  group.add(head)

  const minorGeo = new THREE.BoxGeometry(0.02, 0.02, 0.22)
  const majorGeo = new THREE.BoxGeometry(0.035, 0.035, 0.35)
  const minorMat = new THREE.MeshBasicMaterial({ color: 0x666666 })
  const majorMat = new THREE.MeshBasicMaterial({ color })
  const perp = pickPerp(dirN)
  for (let i = tickEvery; i < Math.floor(length - headLength); i += tickEvery) {
    const isMajor = i % majorEvery === 0
    const g = isMajor ? majorGeo : minorGeo
    const m = isMajor ? majorMat : minorMat
    const tick = new THREE.Mesh(g, m)
    const q = new THREE.Quaternion().setFromUnitVectors(new THREE.Vector3(0,0,1), perp)
    tick.quaternion.copy(q)
    const pos = origin.clone().add(dirN.clone().multiplyScalar(i)).add(perp.clone().multiplyScalar(isMajor ? 0.22 : 0.16))
    tick.position.copy(pos)
    group.add(tick)

    if (isMajor) {
      const nSprite = createTextSprite(String(i), { color: '#222', bg: 'rgba(255,255,255,0.9)', fontSize: 30, padding: 6, radius: 8, bold: false })
      const numPos = pos.clone().add(numberOffset)
      nSprite.position.copy(numPos)
      group.add(nSprite)
    }
  }

  const lbl = createTextSprite(label, { color: '#fff', bg: new THREE.Color(color).getStyle(), fontSize: 40, padding: 10, radius: 10, bold: true })
  const endPos = origin.clone().add(dirN.clone().multiplyScalar(length + 0.2))
  lbl.position.copy(endPos)
  group.add(lbl)
  return group
}
function pickPerp(dir: THREE.Vector3) {
  const up = Math.abs(dir.y) < 0.9 ? new THREE.Vector3(0,1,0) : new THREE.Vector3(1,0,0)
  return new THREE.Vector3().crossVectors(dir, up).normalize()
}

/** ★ 自定义“矩形棋盘”，精确覆盖 [0..x]×[0..y]，步长=1，含 5 的加粗线 */
function buildFloorGridRect(limit: Limit): THREE.LineSegments {
  const X = Math.max(0, limit.x)
  const Y = Math.max(0, limit.y)
  const positions: number[] = []
  const colors: number[] = []

  const cMinor = new THREE.Color(0xCFCFD3)
  const cMajor = new THREE.Color(0x9EA3AA)

  // 竖线：x = 0..X
  for (let x = 0; x <= X; x++) {
    const major = (x % 5 === 0)
    const c = major ? cMajor : cMinor
    positions.push(x, 0, 0,  x, 0, Y)
    colors.push(c.r, c.g, c.b,  c.r, c.g, c.b)
  }
  // 横线：z = 0..Y
  for (let z = 0; z <= Y; z++) {
    const major = (z % 5 === 0)
    const c = major ? cMajor : cMinor
    positions.push(0, 0, z,  X, 0, z)
    colors.push(c.r, c.g, c.b,  c.r, c.g, c.b)
  }

  const geo = new THREE.BufferGeometry()
  geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3))
  geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3))
  const mat = new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.95 })
  const lines = new THREE.LineSegments(geo, mat)
  // 轻微下移避免与体素底面 z-fighting
  lines.position.y = -0.01
  return lines
}

/** 构建体素与边框，并添加坐标轴 + 矩形棋盘 */
function buildVoxels(collection: number[][][], limit: Limit, fitCamera = false) {
  if (!scene) return

  // 清理旧物
  if (voxelGroup) scene.remove(voxelGroup)
  if (gridHelper) scene.remove(gridHelper)
  if (axisGroup) scene.remove(axisGroup)
  if (floorGrid) scene.remove(floorGrid)
  voxelGroup = null
  gridHelper = null
  axisGroup = null
  floorGrid = null
  indexToMeshes.clear()
  setHoveredIndex(null)

  const NZ = collection.length
  if (NZ === 0) return
  const NX = collection[0]?.length ?? 0
  const NY = collection[0]?.[0]?.length ?? 0
  if (NX === 0 || NY === 0) return

  voxelGroup = new THREE.Group()

  for (let zIndex = 0; zIndex < NZ; zIndex++) {
    for (let xIndex = 0; xIndex < NX; xIndex++) {
      for (let yIndex = 0; yIndex < NY; yIndex++) {
        const v = collection[zIndex][xIndex][yIndex]
        if (v <= -1) continue
        const material = getMaterialForValue(v)
        const cube = new THREE.Mesh(boxGeometry, material)
        cube.position.set(xIndex + 0.5, zIndex + 0.5, yIndex + 0.5)
        cube.userData = { idx: v }
        voxelGroup.add(cube)
        let list = indexToMeshes.get(v)
        if (!list) { list = []; indexToMeshes.set(v, list) }
        list.push(cube)
      }
    }
  }
  scene!.add(voxelGroup)

  // 边框： [0, x] × [0, z] × [0, y]
  const gridBox = new THREE.Box3(
      new THREE.Vector3(0, 0, 0),
      new THREE.Vector3(limit.x, limit.z, limit.y)
  )
  gridHelper = new THREE.Box3Helper(gridBox, 0x3a3a3a)
  scene!.add(gridHelper)

  // ★ 矩形底部棋盘（与边框横向切片完全一致）
  floorGrid = buildFloorGridRect(limit)
  scene!.add(floorGrid)

  // 美化坐标轴
  addFancyAxes(limit)

  // 相机贴合
  if (fitCamera && camera) {
    const center = getLimitCenter(limit)
    const size = gridBox.getSize(new THREE.Vector3())
    const maxDim = Math.max(size.x, size.y, size.z || 1)
    const fitHeightDistance = maxDim / (2 * Math.tan(THREE.MathUtils.degToRad(camera.fov) / 2))
    const distance = fitHeightDistance * 1.3
    camera.position.set(center.x + distance, center.y + distance, center.z + distance)
    camera.lookAt(center)
    if (controls) { controls.target.copy(center); controls.update() }
  }
}

function setHoveredIndex(idx: number | null) {
  if (hoveredIndex !== null) indexToMeshes.get(hoveredIndex)?.forEach(m => m.scale.set(1, 1, 1))
  hoveredIndex = idx
  if (hoveredIndex !== null) indexToMeshes.get(hoveredIndex)?.forEach(m => m.scale.set(1.12, 1.12, 1.12))
}

function updateHoverTooltip(idx: number | null, clientX: number, clientY: number) {
  const container = containerRef.value!
  const rect = container.getBoundingClientRect()
  if (idx === null) { tooltip.visible = false; return }
  const name = props.names[idx] ?? `#${idx}`
  const value = props.valuesMap[name]
  tooltip.text = value !== undefined ? `${name}｜价值 ${value}` : `${name}`
  tooltip.x = Math.min(clientX - rect.left + 12, rect.width - 8)
  tooltip.y = Math.min(clientY - rect.top + 12, rect.height - 8)
  tooltip.visible = true
}

function handlePointer(e: MouseEvent) {
  if (!camera || !renderer || !voxelGroup) return
  const container = containerRef.value!
  const rect = container.getBoundingClientRect()
  mouseNdc.x = ((e.clientX - rect.left) / rect.width) * 2 - 1
  mouseNdc.y = -((e.clientY - rect.top) / rect.height) * 2 + 1
  raycaster.setFromCamera(mouseNdc, camera)
  const hits = raycaster.intersectObjects(voxelGroup.children, false)
  if (hits.length > 0) {
    const mesh = hits[0].object as THREE.Mesh
    const idx = (mesh.userData?.idx ?? null) as number | null
    if (idx !== hoveredIndex) setHoveredIndex(idx)
    updateHoverTooltip(idx, e.clientX, e.clientY)
  } else {
    if (hoveredIndex !== null) setHoveredIndex(null)
    tooltip.visible = false
  }
}
function handleMouseLeave() {
  if (hoveredIndex !== null) setHoveredIndex(null)
  tooltip.visible = false
}

function initScene() {
  const container = containerRef.value
  if (!container) return
  const width = container.clientWidth
  const height = container.clientHeight

  scene = new THREE.Scene()
  scene.background = new THREE.Color(0xf3f5f7)

  renderer = new THREE.WebGLRenderer({ antialias: true })
  renderer.setPixelRatio(window.devicePixelRatio)
  renderer.setSize(width, height)
  container.appendChild(renderer.domElement)

  camera = new THREE.PerspectiveCamera(60, width / height, 0.1, 1000)
  scene.add(camera)

  scene.add(new THREE.AmbientLight(0xffffff, 0.55))
  const dir = new THREE.DirectionalLight(0xffffff, 0.9)
  dir.position.set(5, 8, 10)
  scene.add(dir)

  buildVoxels(props.objectCollection, props.limit, true)

  controls = new OrbitControls(camera, renderer.domElement)
  controls.enablePan = false
  controls.autoRotate = true
  controls.autoRotateSpeed = 0.1
  const center = getLimitCenter(props.limit)
  controls.target.copy(center)
  controls.update()

  container.addEventListener('mousemove', handlePointer)
  container.addEventListener('mouseleave', handleMouseLeave)

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

watch(() => props.objectCollection, (val) => {
  if (!scene) return
  buildVoxels(val, props.limit, false)
}, { deep: true })

watch(() => props.limit, (val) => {
  if (!scene) return
  buildVoxels(props.objectCollection, val, false)
  if (camera && controls) {
    const center = getLimitCenter(val)
    camera.lookAt(center)
    controls.target.copy(center)
    controls.update()
  }
}, { deep: true })

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize)
  const container = containerRef.value
  container?.removeEventListener('mousemove', handlePointer)
  container?.removeEventListener('mouseleave', handleMouseLeave)

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
  floorGrid = null
})
</script>

<style scoped>
.three-container {
  position: relative;
  width: 100%;
  height: 100%;
  border: 1px solid #e7eaee;
  overflow: hidden;
  cursor: default;
}
.hover-tooltip {
  position: absolute;
  max-width: 260px;
  padding: 6px 10px;
  font-size: 12px;
  line-height: 1.3;
  color: #fff;
  background: rgba(0,0,0,0.78);
  border-radius: 6px;
  pointer-events: none;
  white-space: nowrap;
  user-select: none;
  box-shadow: 0 6px 18px rgba(0,0,0,0.18);
}
</style>
