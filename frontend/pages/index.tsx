import Head from "next/head";

export default function Home() {
  return (
    <>
      <Head>
        <title>Sceideal</title>
        <meta
          name="description"
          content="A service-focused open source appointment scheduler."
        />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <main>
        <div className="p-2 center-vertically">
          <div className="panel small-panel is-primary">
            <p className="panel-heading">Schedule a CPT session</p>
            <div className="panel-block">
              Appointment Type:
              <div className="select pl-3">
                <select className="is-disabled"></select>
              </div>
            </div>
            <div className="panel-block is-justify-content-end">
              <a className="button is-primary">Next</a>
            </div>
          </div>
        </div>
      </main>
    </>
  );
}
