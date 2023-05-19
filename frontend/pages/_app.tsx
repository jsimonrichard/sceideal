import { ProvideAuthContext, ProvideConfigContext } from "@/components/hooks";
import { NavBar } from "@/components/navbar";
import "@/styles/globals.scss";
import type { AppProps } from "next/app";
import { NextPage } from "next/types";
import { ReactElement, ReactNode } from "react";

export type NextPageWithLayout<P = {}, IP = P> = NextPage<P, IP> & {
  getLayout?: (page: ReactElement) => ReactNode;
};

type AppPropsWithLayout = AppProps & {
  Component: NextPageWithLayout;
};

export default function App({ Component, pageProps }: AppPropsWithLayout) {
  const getLayout =
    Component.getLayout ??
    ((page) => <main className="container">{page}</main>);

  return (
    <ProvideConfigContext>
      <ProvideAuthContext>
        <>
          <NavBar />
          {getLayout(<Component {...pageProps} />)}
        </>
      </ProvideAuthContext>
    </ProvideConfigContext>
  );
}
