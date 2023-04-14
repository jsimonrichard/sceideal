import { AsyncStatus, useAuth } from "@/components/hooks";
import classNames from "classnames";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

function Overview() {
  return <div></div>;
}

function Calendar() {
  return <div></div>;
}

function AppointmentSettings() {
  return <div></div>;
}

function Profile() {
  return <div></div>;
}

function Account() {
  return <div></div>;
}

enum GeneralPanels {
  Overview = "Overview",
  Calendar = "Calendar",
}

enum SettingsPanels {
  AppointmentSettings = "Appointment Settings",
  Profile = "Profile",
  Account = "Account",
}

type PanelName = GeneralPanels | SettingsPanels;

const panels: { [key in PanelName]: () => JSX.Element } = {
  [GeneralPanels.Overview]: Overview,
  [GeneralPanels.Calendar]: Calendar,
  [SettingsPanels.AppointmentSettings]: AppointmentSettings,
  [SettingsPanels.Profile]: Profile,
  [SettingsPanels.Account]: Account,
};

function Dashboard() {
  const router = useRouter();
  const { user, initialLoadStatus } = useAuth();
  useEffect(() => {
    if (!user && initialLoadStatus == AsyncStatus.Error) {
      router.push("/login");
    }
  }, []);

  const [activePanelName, setActivePanelName] = useState<PanelName>(
    GeneralPanels.Overview
  );
  useEffect(() => {
    router.push({ hash: activePanelName }, undefined, { shallow: true });
  }, [activePanelName, router.asPath]);

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
      <aside className="menu column is-one-third is-one-quarter-widescreen is-one-fifth-fullhd p-4">
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
