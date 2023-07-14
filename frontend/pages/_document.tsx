import { Html, Head, Main, NextScript } from "next/document";

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <title>Sceideal</title>
      </Head>
      <body className="has-navbar-fixed-top">
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}
