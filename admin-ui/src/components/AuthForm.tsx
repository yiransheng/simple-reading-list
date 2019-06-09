import * as React from 'react';
import {AuthData, Callback} from '../interface';
import {Input} from './Input';

export interface Props {
  onSubmit: Callback<AuthData>;
}

const { useRef } = React;

export function AuthForm(props: Props): React.ReactElement {
  const emailRef = useRef(null);
  const passwordRef = useRef(null);

  const { onSubmit } = props;

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    const {value:email} = emailRef.current || {value:''};
    const {value:password} = passwordRef.current || {value:''};
    if (email && password) {
      onSubmit({ email, password });
    }
  };

  return <div> 
    <form onSubmit={handleSubmit}>
      <span>Email</span>
      <Input controlled={false} ref={emailRef}/>
      <span>Password</span>
      <input ref={passwordRef} type="password"/>
      <button type="submit">Login</button>
    </form>
  </div>
}

