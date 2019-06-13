import React from 'react';
import {connect} from 'react-redux';

import {match} from './utils';
import {selectState, selectIsLoading} from './selectors';
import {State} from './reducers';
import {State as SyncState} from './state';
import {Variant} from './interface';
import {AuthForm} from './components/AuthForm';

import './styles/css/layout.css';

interface Props {
  state: SyncState;

  isLoading: boolean;
}

const withStoreState = connect((state: State) => ({
  state: selectState(state),
  isLoading: selectIsLoading(state),
}));

const App = withStoreState(({state, isLoading}: Props) => {
  return (
    <div className="container">
      {match(state, {
        annoymous: () => (
          <>
            <h1>Admin Login</h1>
            <div style={{width: '24rem'}}>
              <AuthForm onSubmit={data => console.log(data)} />
            </div>
          </>
        ),
        admin: () => null,
      })}
    </div>
  );
});

export default App;
