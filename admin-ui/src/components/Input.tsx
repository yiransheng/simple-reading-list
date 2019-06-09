import * as React from 'react';

import {Callback} from '../interface';

export interface ControlledProps {
  controlled: true;
  value: string;
  onChange: Callback<string>; 
  type?: string;
}

export interface UncontrolledProps {
  controlled: false;
  type?: string;
}

export type Props = ControlledProps | UncontrolledProps;

export const Input = React.forwardRef((props: Props, ref: React.Ref<HTMLInputElement>) => {
  if (props.controlled) {
    const { value, onChange } = props;
    return <input value={value} onChange={e => {onChange(e.target.value || '');}} type={props.type}/>;
  } else {
    return <input ref={ref} type={props.type}/>;
  }
});
