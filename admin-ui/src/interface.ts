export interface Tagged {
  tag: string;
  value: any;
}

export type Variant<S extends Tagged, T> = S extends {tag: T; value: infer V}
  ? V
  : never;

export type Callback<T> = (arg: T) => void;

export type Result<T, E> = ROk<T> | RErr<E>;

interface ROk<T> {
  tag: 'Ok';
  value: T;
}

interface RErr<T> {
  tag: 'Err';
  value: T;
}

export function Ok<T, E>(value: T): Result<T, E> {
  return {
    tag: 'Ok',
    value,
  };
}
export function Err<T, E>(value: E): Result<T, E> {
  return {
    tag: 'Err',
    value,
  };
}

export type Option<T> = OptionSome<T> | OptionNone<T>;

export function Some<T>(value: T): Option<T> {
  return {tag: 'Some', value};
}
export function None<T>(): Option<T> {
  return {tag: 'None'};
}

interface OptionSome<T> {
  tag: 'Some';
  value: T;
}

interface OptionNone<T> {
  tag: 'None';
}

export interface AuthData {
  email: string;
  password: string;
}

export interface AdminUser {
  email: string;
}

export type AuthResult = Result<AuthSuccess, GenericError>;

export interface AuthSuccess {
  user: AdminUser;
  token: string;
}

// wraps in an object, so JSON.stringify works mostly
export interface GenericError {
  error: unknown;
}

export interface Bookmark {
  title: string;
  url: string;
  body: string;
  tags: string[];
}
