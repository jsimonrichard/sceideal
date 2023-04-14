import { ProvideAuthContext } from "@/components/hooks";
import { NavBar } from "@/components/navbar";
import "@/styles/globals.scss";
import type { AppProps } from "next/app";

export default function App({ Component, pageProps }: AppProps) {
  return (
    <ProvideAuthContext>
      <>
        <NavBar />
        <main className="container">
          <Component {...pageProps} />
        </main>
      </>
    </ProvideAuthContext>
  );
}
