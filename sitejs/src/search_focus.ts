const inputSelector = "[name = q]";

function moveCursorToEnd(el: any) {
  if (typeof el.selectionStart === "number") {
    el.selectionStart = el.selectionEnd = el.value.length;
  } else if (typeof el.createTextRange !== "undefined") {
    el.focus();
    const range = el.createTextRange();
    range.collapse(false);
    range.select();
  }
}

export function main() {
  const input = document.querySelector(inputSelector);
  if (input instanceof HTMLInputElement) {
    input.focus();
    moveCursorToEnd(input);
  }
}
