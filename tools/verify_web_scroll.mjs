// Real WebGPU/input cycles in a desktop browser. No mobile visual validation.
// Serve web/ first; install the existing tools/reference-app dependencies.
import { chromium } from './reference-app/node_modules/playwright/index.mjs'
import assert from 'node:assert/strict'

const origin = process.env.GALLERY_URL || 'http://127.0.0.1:8080/'
const browser = await chromium.launch({headless: false, args: ['--enable-unsafe-webgpu']})
try {
  const page = await browser.newPage({viewport: {width: 1200, height: 800}, hasTouch: true})
  const errors = []
  page.on('pageerror', error => errors.push(String(error)))
  const cdp = await page.context().newCDPSession(page)
  async function route(query) {
    await page.goto(`${origin}?${query}`)
    await page.waitForFunction(() => document.getElementById('status').hidden, null, {timeout: 60000})
  }
  const offset = () => page.locator('#page-scroll').evaluate(element => element.scrollTop)
  async function swipe(x, start, end) {
    await cdp.send('Input.dispatchTouchEvent', {type: 'touchStart', touchPoints: [{x, y: start}]})
    for (let i = 1; i <= 5; ++i) {
      await cdp.send('Input.dispatchTouchEvent', {type: 'touchMove', touchPoints: [{x, y: start + (end - start) * i / 5}]})
      await page.waitForTimeout(16)
    }
    await cdp.send('Input.dispatchTouchEvent', {type: 'touchEnd', touchPoints: []})
  }

  await route('component=button')
  await page.keyboard.press('PageDown')
  await page.waitForFunction(() => document.getElementById('page-scroll').scrollTop > 100)
  await page.locator('#page-scroll').evaluate(element => { element.scrollTop = 0 })
  await page.waitForTimeout(200)
  await page.mouse.move(700, 500)
  await page.mouse.wheel(0, 250)
  await page.waitForFunction(() => document.getElementById('page-scroll').scrollTop > 200)
  await page.locator('#page-scroll').evaluate(element => { element.scrollTop = 0 })
  await page.waitForTimeout(150)
  await swipe(700, 650, 400)
  const released = await offset()
  await page.waitForTimeout(300)
  const settled = await offset()
  assert(released > 0, 'touch moves the native scroll container')
  assert(settled > released + 10, 'native momentum continues after release')
  await page.mouse.click(250, 75) // Settings
  await page.waitForTimeout(200)
  assert.equal(await offset(), 0, 'route navigation resets the browser position')

  await route('showcase=checkout')
  await cdp.send('Input.dispatchTouchEvent', {type: 'touchStart', touchPoints: [{x: 180, y: 260}]})
  await cdp.send('Input.dispatchTouchEvent', {type: 'touchEnd', touchPoints: []})
  await page.waitForTimeout(100)
  assert.equal(await page.evaluate(() => document.activeElement.id), 'keyboard')
  await page.keyboard.type('Browser scroll')
  await page.keyboard.press('Backspace')

  await route('component=slider')
  // Find the actual visible vertical control, scrolling its real gallery page.
  let vertical
  for (let i = 0; i < 12 && !vertical; ++i) {
    vertical = await page.locator('.drag-input').evaluateAll(elements => elements
      .map(element => element.getBoundingClientRect().toJSON())
      .find(rect => rect.height > 80 && rect.height > rect.width))
    if (!vertical) {
      await page.locator('#page-scroll').evaluate(element => { element.scrollTop += 250 })
      await page.waitForTimeout(50)
    }
  }
  assert(vertical, 'the vertical slider has a browser drag region')
  const beforeDrag = await offset()
  await page.evaluate(() => {
    window.cancelledPointers = 0
    document.getElementById('viewport').addEventListener('pointercancel', () => ++window.cancelledPointers)
  })
  await swipe(vertical.x + vertical.width / 2, vertical.y + vertical.height * .75, vertical.y + vertical.height * .25)
  await page.waitForTimeout(100)
  assert.equal(await offset(), beforeDrag, 'dragging the slider does not pan the page')
  assert.equal(await page.evaluate(() => window.cancelledPointers), 0, 'the browser does not steal a slider drag')

  await route('component=dialog')
  await page.mouse.click(700, 397)
  await page.waitForFunction(() => !document.getElementById('overlay-input').hidden)
  const beforeOverlay = await offset()
  await page.mouse.move(1100, 650)
  await page.mouse.wheel(0, 250)
  await page.waitForTimeout(100)
  assert.equal(await offset(), beforeOverlay, 'an overlay prevents background page scrolling')
  await page.keyboard.press('Escape')
  await page.waitForFunction(() => document.getElementById('overlay-input').hidden)
  assert.deepEqual(errors, [])
  console.log(`Passed: native Page Down, wheel, touch momentum (${released} → ${settled}), route reset, editor focus, slider drag and overlay dismissal.`)
} finally {
  await browser.close()
}
