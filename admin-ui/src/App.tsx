import React from 'react';
import logo from './logo.svg';
import './App.css';

import {AuthForm} from './components/AuthForm';

const App: React.FC = () => {
  return (
    <div className="App">
      <AuthForm onSubmit={data => console.log(data)}/> 
    </div>
  );
}

export default App;
