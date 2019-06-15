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
import {signin, signout} from './action_creators';

import './styles/css/layout.css';

interface Props {
  state: SyncState;

  isLoading: boolean;

  handleSignIn: (data: AuthData) => void;

  handleSignOut: () => void;
}

const withStoreState = connect(
  (state: State) => ({
    state: selectState(state),
    isLoading: selectIsLoading(state)
  }),
  dispatch => bindActionCreators({
    handleSignIn: signin,
    handleSignOut: signout,
  }, dispatch)
);

const App = withStoreState(({state, isLoading, handleSignIn, handleSignOut}: Props) => {
  return (
    <div className="container">
      <LoadingIndicator show={isLoading} msg="sending request..." />
      {match(state, {
        unknown: () => null,
        annoymous: () => (
          <>
            <h1>Admin Login</h1>
            <div style={{width: '24rem'}}>
              <AuthForm onSubmit={handleSignIn} />
            </div>
          </>
        ),
        admin: () => <button onClick={handleSignOut}>Logout</button>,
      })}
    </div>
  );
});

export default App;
