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
  value?: string;
  type?: string;
  placeholder?: string;
}

export type Props = ControlledProps | UncontrolledProps;

interface State {
  local: string;
  commited: boolean;
}

export const Input = React.forwardRef(
  (props: Props, ref: React.Ref<HTMLInputElement>) => {
    const [st, setSt] = React.useState({
      local: props.value || '',
      commited: false,
    });

    let input;
    if (props.controlled) {
      const {value, onChange, type = 'text', placeholder} = props;
      if (st.commited && st.local !== value) {
        setSt({ commited: true, local: value });
      }

      input = (
        <input
          value={st.commited ? value : st.local}
          onFocus={() => setSt(st => ({...st, commited: false}))}
          onChange={e => {
            const local = e.target.value;
            setSt(st => ({...st, local}));
          }}
          onBlur={() => {
            setSt(st => ({...st, commited: true}));
            onChange(st.local);
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
