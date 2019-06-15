import React from 'react';
import {connect} from 'react-redux';
import {bindActionCreators} from 'redux';

import {match} from './utils';
import {selectState, selectIsLoading} from './selectors';
import {State} from './reducers';
import {State as SyncState} from './state';
import {Variant, AuthData} from './interface';
import {AuthForm} from './components/AuthForm';
import {LoadingIndicator} from './components/LoadingIndicator';
import {Dispatchable} from './actions';
import {signin} from './action_creators';

import './styles/css/layout.css';

interface Props {
  state: SyncState;

  isLoading: boolean;

  handleSubmit: (data: AuthData) => void,
}

const withStoreState = connect(
  (state: State) => ({
    state: selectState(state),
    isLoading: selectIsLoading(state)
  }),
  dispatch => bindActionCreators({
    handleSubmit: signin,
  }, dispatch)
);

const App = withStoreState(({state, isLoading, handleSubmit}: Props) => {
  return (
    <div className="container">
      <LoadingIndicator show={isLoading} msg="sending request..." />
      {match(state, {
        annoymous: () => (
          <>
            <h1>Admin Login</h1>
            <div style={{width: '24rem'}}>
              <AuthForm onSubmit={handleSubmit} />
            </div>
          </>
        ),
        admin: () => <h2>Ok</h2>,
      })}
    </div>
  );
});

export default App;
