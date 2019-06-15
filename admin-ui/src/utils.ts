import {GenericError, Tagged, Variant} from './interface';
import {Reducer, Action} from 'redux';
import {AppStatus} from './state';

export interface SubstateReducer<S extends Tagged, T extends string, A> {
  (state: Variant<S, T>, action: A): S;
}

type SubstateReducerMap<S extends Tagged, A> = {
  [Tag in S['tag']]: SubstateReducer<S, Tag, A>;
};

export interface StrictReducer<S, A> {
  (state: S, action: A): S;
}

export function createStateMachineReducer<S extends Tagged, A extends Action>(
  mapping: SubstateReducerMap<S, A>,
): StrictReducer<S, A> {
  return function(state: S, action: A) {
    const innerReducer = mapping[state.tag as keyof SubstateReducerMap<S, A>];

    if (innerReducer) {
      return innerReducer(state.value, action);
    } else {
      return state;
    }
  };
}

export function withInitialState<S, A extends Action>(
  seedState: S,
): (reducer: StrictReducer<S, A>) => Reducer<S, A> {
  return reducer => (state: S | undefined, action: A) => {
    if (state != null) {
      return reducer(state, action);
    } else {
      return seedState;
    }
  };
}

type MatchArms<S extends Tagged, U> = {
  [Tag in S['tag']]?: (value: Variant<S, Tag>) => U;
} & {_?(): U};

export function match<T extends Tagged, U>(expr: T, arms: MatchArms<T, U>): U {
  const {tag, value} = expr;
  const arm = arms[tag as keyof MatchArms<T, U>];
  if (arm) {
    return arm(value);
  } else if (arms._) {
    return arms._();
  } else {
    throw new Error('No Match');
  }
}

let id = 0;
export function uid(): number {
  return ++id;
}

export function getErrorStatus(err: GenericError, when: Date): AppStatus {
  return {
    tag: 'err',
    value: {
      // TODO: to config file
      dismissWhen: new Date(when.getTime() + 10000),
      message: getErrorMessage(err),
    },
  };
}

export function getErrorMessage(err: GenericError): string {
  const {error} = err;
  if (typeof error === 'string') {
    return error;
  } else {
    console.error('[ERROR]', error);
    return 'Generic error, see console.';
  }
}
