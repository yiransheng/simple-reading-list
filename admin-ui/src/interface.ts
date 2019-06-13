export interface Tagged {
  tag: string;
  value: any;
}

export type Variant<S extends Tagged, T> = S extends { tag: T; value: infer V }
  ? V
  : never;

export type Callback<T> = (arg: T) => void;

export type Result<T, E> = Ok<T> | Err<E>;

interface Ok<T> {
  tag: "ok";
  value: T;
}

interface Err<T> {
  tag: "err";
  value: T;
}

export type Option<T> = OptionSome<T> | OptionNone<T>;

export function Some<T>(value: T): Option<T> {
  return { tag: "some", value };
}
export function None<T>(): Option<T> {
  return { tag: "none" };
}

interface OptionSome<T> {
  tag: "some";
  value: T;
}

interface OptionNone<T> {
  tag: "none";
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

export interface Bookmark {
  title: string;
  url: string;
  body: string;
  tags: string[];
}
