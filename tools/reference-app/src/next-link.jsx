// Native browser anchor for the isolated source harness; no Next router is needed.
import * as React from 'react'
export default React.forwardRef(function Link({href,children,...props},ref) {
    return <a ref={ref} href={href} {...props}>{children}</a>
})
