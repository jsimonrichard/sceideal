import axios, { AxiosError, AxiosResponse } from "axios";
import { redirect } from "next/dist/server/api-utils";
import { useRouter } from "next/router";
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import { CreateUser, LoginData, UserData } from "../shared-types";

export enum AsyncStatus {
  Idle,
  Pending,
  Success,
  Error,
}

// https://usehooks.com/useAsync/
export const useAsync = <I, T, E>(
  asyncFunction: (data?: I) => Promise<T>,
  onSuccess?: (response: T) => void,
  onFailure?: (error: E) => void,
  immediate = false,
  data?: I
) => {
  const [status, setStatus] = useState(AsyncStatus.Idle);
  const [value, setValue] = useState<T | null>();
  const [error, setError] = useState<E | null>();

  const execute = useCallback(
    (data?: I) => {
      setStatus(AsyncStatus.Pending);
      setValue(null);
      setError(null);

      return asyncFunction(data)
        .then((response) => {
          setValue(response);
          setStatus(AsyncStatus.Success);
          if (onSuccess) {
            onSuccess(response);
          }
        })
        .catch((error: E) => {
          setError(error);
          setStatus(AsyncStatus.Error);
          if (onFailure) {
            onFailure(error);
          }
        });
    },
    [asyncFunction]
  );

  useEffect(() => {
    if (immediate) {
      execute(data);
    }
  }, []);

  return { execute, status, value, error };
};

const AuthContext = createContext<ReturnType<typeof useProvideAuth>>(
  {} as ReturnType<typeof useProvideAuth>
);

export const useAuth = () => {
  const router = useRouter();
  const auth = useContext(AuthContext);
  return auth;
};

export function ProvideAuthContext({ children }: { children: JSX.Element }) {
  const auth = useProvideAuth();
  return <AuthContext.Provider value={auth}>{children}</AuthContext.Provider>;
}

function useProvideAuth() {
  const router = useRouter();
  const [user, setUser] = useState<UserData | null>();

  const { status: initialLoadStatus } = useAsync<
    null,
    AxiosResponse<UserData>,
    Error | AxiosError
  >(
    () => axios.get<UserData>("/api/user"),
    (response) => {
      setUser(response.data);
    },
    () => {},
    true // immediate evaluation
  );

  const login = (redirectUrl?: string) =>
    useAsync<LoginData, AxiosResponse<UserData>, Error | AxiosError>(
      (data?: LoginData) => axios.post<UserData>("/api/user/login", data),
      (response) => {
        setUser(response.data);
        if (redirectUrl) {
          router.push(redirectUrl);
        }
      }
    );

  const sign_up = (redirectUrl?: string) =>
    useAsync<CreateUser, AxiosResponse<UserData>, Error | AxiosError>(
      (data) => axios.post<UserData>("/api/user/login", data),
      (response) => {
        setUser(response.data);
        if (redirectUrl) {
          router.push(redirectUrl);
        }
      }
    );

  const logout = (redirectUrl?: string) =>
    useAsync<CreateUser, null, Error | AxiosError>(
      () => axios.post("/api/user/logout"),
      () => {
        setUser(null);
        if (redirectUrl) {
          router.push(redirectUrl);
        }
      }
    );

  return { user, login, sign_up, logout, initialLoadStatus };
}
