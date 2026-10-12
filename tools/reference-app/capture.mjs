import { chromium } from 'playwright'
import { mkdir,writeFile } from 'node:fs/promises'
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
const [example='button-default',theme='light',output='/tmp/reference.png',state='default',width='600',height='400',keys='',motionTime='',scaleFactor='1']=process.argv.slice(2)
const browser=await chromium.launch({headless:true})
try {
 const page=await browser.newPage({viewport:{width:Number(width),height:Number(height)},deviceScaleFactor:Number(scaleFactor),colorScheme:theme})
 let animationSamples=[]
 const errors=[];page.on('pageerror',e=>errors.push(String(e)))
 const assetOverrides=[]
 if(example.startsWith('avatar-') || example==='dropdown-menu-avatar') {
  const portraits={shadcn:'shadcn.jpg',maxleiter:'01.png',evilrabbit:'02.png',pranathip:'03.png'}
  await page.route('https://github.com/*.png',async route=>{
   const user=new URL(route.request().url()).pathname.slice(1).replace(/\.png$/,'')
   const asset=portraits[user]
   if(state==='image-error') {await route.abort();return}
   if(!asset) {await route.continue();return}
   // Use the same pinned first-party image assets on both renderers instead of
   // mutable external profile pictures. Component/example source stays intact.
   const path=fileURLToPath(new URL(`../../crates/gallery/assets/avatars/${asset}`,import.meta.url))
   assetOverrides.push({url:route.request().url(),asset:`crates/gallery/assets/avatars/${asset}`})
   await route.fulfill({path,contentType:asset.endsWith('.jpg')?'image/jpeg':'image/png'})
  })
 }
 await page.goto(`${process.env.REFERENCE_URL||'http://127.0.0.1:4173'}/?example=${example}&theme=${theme}`)
 await page.waitForFunction(()=>window.referenceReady===true && document.getElementById('root').childElementCount>0)
 const tooltipArea=example==='tooltip-disabled' || ['button-group-nested','button-group-input-group'].includes(example)
 const target=page.locator(example.startsWith('slider-')?'[data-slot=slider-thumb]':tooltipArea && state==='open'?'[data-slot=tooltip-trigger]':example.startsWith('select-') && state==='open'?'[data-slot=select-trigger]':example.startsWith('button-group-') && state==='open'?'button[aria-haspopup]':'button, input, textarea, [tabindex]').filter({visible:true}).first()
 if(await target.count()) await target.waitFor({state:'visible'})
 await page.evaluate(()=>document.fonts.ready)
 if(example.startsWith('slider-')) {
  const track=await page.locator('[data-slot=slider-track]').first().boundingBox()
  if(!track || track.width<=0 || track.height<=0) throw new Error('Slider track is empty; check the upstream shadcn.css orientation variants')
 }
 await page.locator('[data-slot="avatar-image"]').evaluateAll(images=>Promise.all(images.map(image=>image.decode().catch(()=>{}))))
 // Pause Chromium's animation clock before input so short transitions survive
 // until sampling. Source components and their timing curves remain unchanged.
 if(motionTime!=='') {
  await page.waitForTimeout(150)
  const session=await page.context().newCDPSession(page)
  await session.send('Animation.enable')
  await session.send('Animation.setPlaybackRate',{playbackRate:0})
 }
 if(state==='hover' && await target.count()) await target.hover()
 if(state==='focus') await page.keyboard.press('Tab')
 if(state==='disabled' && await target.count()) await target.evaluate(el=>{el.disabled=true})
 if(state==='open' && (example.startsWith('tooltip-') || tooltipArea) && await target.count()) await target.hover()
 else if((state==='selected'||state==='open') && await target.count()) await target.click()
 // Base UI transfers focus into newly mounted popups after their first layout.
 // Let that transfer settle before sending the first navigation key; otherwise
 // it can land on the trigger and be consumed by opening rather than navigation.
 if(keys) await page.waitForTimeout(150)
 for(const key of keys.split(',').filter(Boolean)) {
  if(motionTime!=='') await page.evaluate(async()=>{
   // Each native scripted key settles its preceding action before dispatch.
   // Finish finite browser transitions too, then sample only the latest action.
   for(const animation of document.getAnimations()) if(Number.isFinite(animation.effect.getTiming().iterations)) animation.finish()
   await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))
  })
  await page.keyboard.press(key);await page.waitForTimeout(120)
 }
 if(motionTime!=='') {
  animationSamples=await page.evaluate(async time=>{
   await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))
   const animations=document.getAnimations()
   for(const animation of animations) {animation.pause();animation.currentTime=Number(time)}
   await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))
   return animations.map(animation=>({property:animation.transitionProperty||animation.animationName,currentTime:animation.currentTime,timing:animation.effect.getTiming()}))
  },motionTime)
 } else await page.waitForTimeout(example==='progress-demo'?650:150)
 if(errors.length) throw new Error(errors.join('\n'))
 await mkdir(dirname(output),{recursive:true})
 await page.screenshot({path:output,animations:motionTime===''?'disabled':'allow'})
 const bounds=await target.count()?await target.boundingBox():await page.locator('#root').boundingBox()
 await writeFile(`${output}.json`,JSON.stringify({example,theme,state,keys:keys.split(',').filter(Boolean),assetOverrides,width:Number(width),height:Number(height),deviceScaleFactor:Number(scaleFactor),animationSamples,motionTimeMs:motionTime===''?null:Number(motionTime),bounds},null,2)+'\n')
 console.log(output)
} finally {await browser.close()}
