import React from 'react';
import logo from './logo.svg';

import {AuthForm} from './components/AuthForm';

const App: React.FC = () => {
  return (
    <div style={{width: 500}}>
      <AuthForm onSubmit={data => console.log(data)} />
    </div>
  );
};

export default App;
