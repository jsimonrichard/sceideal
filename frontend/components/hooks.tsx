import axios, { AxiosError, AxiosResponse } from "axios";
import { useRouter } from "next/router";
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from "react";
import { CreateUser, LoginData, PublicConfig, UserData } from "../shared-types";

export enum AsyncStatus {
  Idle,
  Pending,
  Success,
  Error,
}

// https://usehooks.com/useAsync/
export const useAsync = <I, T>(
  asyncFunction: (data?: I) => Promise<AxiosResponse<T>>,
  onSuccess?: (response: AxiosResponse<T>) => void,
  onFailure?: (error: Error | AxiosError) => void,
  immediate = false,
  data?: I
) => {
  const [status, setStatus] = useState(AsyncStatus.Idle);
  const [value, setValue] = useState<T | null>();
  const [error, setError] = useState<Error | AxiosError | null>();

  const execute = useCallback(
    (data?: I) => {
      setStatus(AsyncStatus.Pending);
      setValue(null);
      setError(null);

      return asyncFunction(data)
        .then((response) => {
          setValue(response.data);
          setStatus(AsyncStatus.Success);
          if (onSuccess) {
            onSuccess(response);
          }
        })
        .catch((error: Error | AxiosError) => {
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

export const useAuth = (isProtected = false) => {
  const auth = useContext(AuthContext);

  const router = useRouter();
  useEffect(() => {
    if (isProtected) {
      console.log(auth.initialLoadStatus);
    }
    if (
      isProtected &&
      !auth.user &&
      auth.initialLoadStatus == AsyncStatus.Error
    ) {
      router.push("/login");
    }
  }, [auth]);

  return auth;
};

export function ProvideAuthContext({ children }: { children: JSX.Element }) {
  const auth = useProvideAuth();
  return <AuthContext.Provider value={auth}>{children}</AuthContext.Provider>;
}

function useProvideAuth() {
  const router = useRouter();
  const [user, setUser] = useState<UserData | null>();

  const { status: initialLoadStatus } = useAsync<null, UserData>(
    () => axios.get<UserData>("/api/user"),
    (response) => {
      setUser(response.data);
    },
    () => {},
    true // immediate evaluation
  );

  const login = (redirectUrl?: string) =>
    useAsync<LoginData, UserData>(
      (data?: LoginData) => axios.post<UserData>("/api/user/login", data),
      (response) => {
        setUser(response.data);
        if (redirectUrl) {
          router.push(redirectUrl);
        }
      }
    );

  const sign_up = (redirectUrl?: string) =>
    useAsync<CreateUser, UserData>(
      (data) => axios.post<UserData>("/api/user/login", data),
      (response) => {
        setUser(response.data);
        if (redirectUrl) {
          router.push(redirectUrl);
        }
      }
    );

  const logout = () =>
    useAsync<CreateUser, string>(
      () => axios.post("/api/user/logout"),
      (response) => {
        setUser(null);
        if (response.data && response.data.length > 0) {
          router.push(response.data);
        } else {
          router.push("/");
        }
      }
    );

  return { user, login, sign_up, logout, initialLoadStatus };
}

export const useConfig = () => {
  const ctx = useContext(ConfigContext);
  return ctx;
};

const ConfigContext = createContext<ReturnType<typeof useProvideConfig>>(
  {} as ReturnType<typeof useProvideConfig>
);

export function ProvideConfigContext({ children }: { children: JSX.Element }) {
  const config = useProvideConfig();
  return (
    <ConfigContext.Provider value={config}>{children}</ConfigContext.Provider>
  );
}

function useProvideConfig() {
  const [config, setConfig] = useState<PublicConfig | null>();

  const { status: initialLoadStatus } = useAsync<null, PublicConfig>(
    () => axios.get<PublicConfig>("/api/config"),
    (response) => {
      setConfig(response.data);
    },
    () => {},
    true // immediate evaluation
  );

  return {
    config,
    initialLoadStatus,
  };
}
