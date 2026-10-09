import { chromium } from 'playwright'
import { mkdir,writeFile } from 'node:fs/promises'
import { dirname } from 'node:path'
const [example='button-default',theme='light',output='/tmp/reference.png',state='default',width='600',height='400',keys='']=process.argv.slice(2)
const browser=await chromium.launch({headless:true})
try {
 const page=await browser.newPage({viewport:{width:Number(width),height:Number(height)},deviceScaleFactor:1,colorScheme:theme})
 const errors=[];page.on('pageerror',e=>errors.push(String(e)))
 await page.goto(`${process.env.REFERENCE_URL||'http://127.0.0.1:4173'}/?example=${example}&theme=${theme}`)
 await page.waitForFunction(()=>window.referenceReady===true && document.getElementById('root').childElementCount>0)
 const target=page.locator('button, input, textarea, [tabindex]').filter({visible:true}).first()
 if(await target.count()) await target.waitFor({state:'visible'})
 await page.evaluate(()=>document.fonts.ready)
 if(state==='hover' && await target.count()) await target.hover()
 if(state==='focus') await page.keyboard.press('Tab')
 if(state==='disabled' && await target.count()) await target.evaluate(el=>{el.disabled=true})
 if((state==='selected'||state==='open') && await target.count()) await target.click()
 for(const key of keys.split(',').filter(Boolean)) {await page.keyboard.press(key);await page.waitForTimeout(120)}
 await page.waitForTimeout(example==='progress-demo'?650:150)
 if(errors.length) throw new Error(errors.join('\n'))
 await mkdir(dirname(output),{recursive:true})
 await page.screenshot({path:output,animations:'disabled'})
 const bounds=await target.count()?await target.boundingBox():await page.locator('#root').boundingBox()
 await writeFile(`${output}.json`,JSON.stringify({example,theme,state,keys:keys.split(',').filter(Boolean),width:Number(width),height:Number(height),deviceScaleFactor:1,bounds},null,2)+'\n')
 console.log(output)
} finally {await browser.close()}
