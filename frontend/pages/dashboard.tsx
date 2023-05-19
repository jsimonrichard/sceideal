import { AsyncStatus, useAuth } from "@/components/hooks";
import { faCirclePlus, faPlus } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import classNames from "classnames";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

function Overview() {
  return <div></div>;
}

function Calendar() {
  return <div></div>;
}

function AppointmentTypes() {
  return (
    <div>
      <h1 className="title is-2">Appointment Types</h1>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(240px, 300px))",
          columnGap: 10,
          rowGap: 10,
        }}
      >
        <div className="block-cell">
          <span>
            Zoom <br />
            <a>asdfasdfasdf</a>
          </span>
        </div>
        <div className="block-cell">
          <span>
            Discord <br />
            <a>asdfasdfasdf</a>
          </span>
        </div>
        <div className="block-cell" style={{ fontSize: "2rem" }}>
          <FontAwesomeIcon icon={faPlus} />
        </div>
      </div>
    </div>
  );
}

function Locations() {
  return (
    <div>
      <h1 className="title is-2">Meeting Locations</h1>
    </div>
  );
}

function Account() {
  return <div></div>;
}

enum GeneralPanels {
  Overview = "Overview",
  Calendar = "Calendar",
}

enum SettingsPanels {
  AppointmentTypes = "Appointment Types",
  Locations = "Locations",
  Account = "Account",
}

type PanelName = GeneralPanels | SettingsPanels;

const panels: { [key in PanelName]: () => JSX.Element } = {
  [GeneralPanels.Overview]: Overview,
  [GeneralPanels.Calendar]: Calendar,
  [SettingsPanels.AppointmentTypes]: AppointmentTypes,
  [SettingsPanels.Locations]: Locations,
  [SettingsPanels.Account]: Account,
};

const URL_UPDATE_DELAY = 100;

function Dashboard() {
  const router = useRouter();
  const { user, initialLoadStatus } = useAuth(true);

  const [activePanelName, setActivePanelName] = useState<PanelName>(
    GeneralPanels.Overview
  );
  useEffect(() => {
    const timeout = setTimeout(() => {
      router.push({ hash: activePanelName }, undefined, { shallow: true });
    }, URL_UPDATE_DELAY);

    return function cleanUp() {
      clearTimeout(timeout);
    };
  }, [activePanelName]);

  // Get panel from url if possible
  const [isLoadingInitPanel, setIsLoadingInitPanel] = useState(true);
  useEffect(() => {
    let id = decodeURI(router.asPath.split("#").reverse()[0]);
    if (
      Object.values(GeneralPanels).includes(id as GeneralPanels) ||
      Object.values(SettingsPanels).includes(id as SettingsPanels)
    ) {
      setActivePanelName(id as PanelName);
    }
    setIsLoadingInitPanel(false);
  }, [router.asPath]);

  const renderPanelButton = (key: PanelName) => (
    <li key={key}>
      <a
        className={classNames({
          "is-active": key == activePanelName,
        })}
        onClick={() => setActivePanelName(key)}
      >
        {key}
      </a>
    </li>
  );

  if (!user || isLoadingInitPanel) {
    return <></>;
  }

  const Panel = panels[activePanelName];

  return (
    <div className="has-navbar-fixed-top columns" style={{ marginTop: "5rem" }}>
      <aside className="menu column is-narrow p-4">
        <p className="menu-label">General</p>
        <ul className="menu-list">
          {Object.values(GeneralPanels).map(renderPanelButton)}
        </ul>
        <p className="menu-label">Settings</p>
        <ul className="menu-list">
          {Object.values(SettingsPanels).map(renderPanelButton)}
        </ul>
      </aside>

      <div className="column">
        <Panel />
      </div>
    </div>
  );
}

export default Dashboard;
