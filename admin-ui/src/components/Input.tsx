import * as React from 'react';

import {Callback} from '../interface';

import '../styles/css/form.css';

export interface ControlledProps {
  controlled: true;
  value: string;
  onChange: Callback<string>;
  label: string;
  type?: string;
  placeholder?: string;
}

export interface UncontrolledProps {
  controlled: false;
  label: string;
  type?: string;
  placeholder?: string;
}

export type Props = ControlledProps | UncontrolledProps;

export const Input = React.forwardRef(
  (props: Props, ref: React.Ref<HTMLInputElement>) => {
    let input;
    if (props.controlled) {
      const {value, onChange, type = 'text', placeholder} = props;
      input = (
        <input
          value={value}
          onChange={e => {
            onChange(e.target.value || '');
          }}
          placeholder={placeholder}
          type={type}
        />
      );
    } else {
      const {type = 'text', placeholder} = props;
      input = <input ref={ref} type={type} placeholder={placeholder} />;
    }

    return (
      <fieldset className="input_fieldset">
        <label>{props.label}</label>
        {input}
      </fieldset>
    );
  },
);
