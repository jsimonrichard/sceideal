import { faArrowLeft } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import Link from "next/link";

export default function GoBackLayout({
  children,
  backTo,
}: {
  children: JSX.Element;
  backTo: string;
}) {
  return (
    <main className="container">
      <div className="columns" style={{ marginTop: "2rem" }}>
        <aside
          className="column is-narrow p-4"
          style={{
            minWidth: "10rem",
            marginTop: "2.5rem",
          }}
        >
          <Link className="button is-inverted is-link" href={backTo}>
            <FontAwesomeIcon icon={faArrowLeft} className="mr-2" />
            Go Back
          </Link>
        </aside>

        <div className="column">{children}</div>
      </div>
    </main>
  );
}
