export type Result<T, E> = Ok<T> | Err<E>;

export type Callback<T> = (arg: T) => void;

export interface Ok<T> {
  tag: "ok";
  value: T;
}

export interface Err<T> {
  tag: "err";
  value: T;
}

export interface AuthData {
  email: string;
  password: string;
}

export type AuthResult = Result<AuthSuccess, AuthError>;

export interface AuthSuccess {
  token: string;
}

export interface AuthError {
  error: unknown;
}

export interface Tagged {
  tag: string;
  value: any;
}

export type Variant<S extends Tagged, T> = S extends { tag: T; value: infer V }
  ? V
  : never;
