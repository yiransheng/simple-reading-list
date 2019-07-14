interface Attributes {
  [key: string]: string
}
type NotObject = string | unknown[]

interface ElOrAttrsList extends Array<Attributes | JsonMl> {}

type JsonMlElement1 = [string, Attributes, ...NotObject[]] & ElOrAttrsList
type JsonMlElement2 = [string, ...NotObject[]] & ElOrAttrsList

export type JsonMl =
  | string // text node
  | JsonMlElement1
  | JsonMlElement2

function isAttrs(attrs: any): attrs is Attributes {
  if (attrs == null) {
    return false
  }
  if (Array.isArray(attrs)) {
    return false
  }
  return typeof attrs === 'object'
}
function isElement1(jsonml: JsonMl): jsonml is JsonMlElement1 {
  if (!Array.isArray(jsonml)) {
    return false
  }
  if (jsonml.length < 2) {
    return false
  }
  return isAttrs(jsonml[1])
}

export function createElement(jsonml: JsonMl): Text | HTMLElement {
  const doc = document

  if (typeof jsonml === 'string') {
    return doc.createTextNode(jsonml)
  }
  if (jsonml.length === 1) {
    return doc.createElement(jsonml[0])
  }
  if (isElement1(jsonml)) {
    const [tag, attrs, ...rest] = jsonml
    const el = doc.createElement(tag)
    for (const [k, v] of Object.entries(attrs)) {
      el.setAttribute(k, v)
    }
    const children = rest as JsonMl[]
    for (const c of children) {
      const child = createElement(c)
      el.appendChild(child)
    }
    return el
  }

  const [tag, ...rest] = jsonml
  const el = doc.createElement(tag)
  const children = rest as JsonMl[]
  for (const c of children) {
    const child = createElement(c)
    el.appendChild(child)
  }

  return el
}
