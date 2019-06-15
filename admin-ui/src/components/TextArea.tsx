import React from "react";

import { Callback } from "../interface";
import "../styles/css/form.css";

export interface Props {
  label: string;
  value: string;
  onChange: Callback<string>;
}

export function TextArea(props: Props): JSX.Element {
  const { label, value, onChange } = props;
  return (
    <fieldset className="textarea_fieldset">
      <label>{label}</label>
      <textarea rows={10} value={value} onChange={e => onChange(e.target.value)} />
    </fieldset>
  );
}
