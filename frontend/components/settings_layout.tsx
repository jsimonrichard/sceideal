import { PermissionLevel } from "@/shared-types";
import classNames from "classnames";
import Link from "next/link";
import { useAuth, AsyncStatus } from "./hooks";

export enum GeneralSettingPage {
  Profile = "profile",
  Account = "account",
}

export enum TeacherSettingPage {
  Availability = "availability",
  Topics = "topics",
  AppointmentTypes = "appointment-types",
  Locations = "locations",
}

export enum AdminSettingPage {
  Users = "users",
  Classes = "classes",
}

type SettingsPage = GeneralSettingPage | TeacherSettingPage | AdminSettingPage;

const PAGE_TITLES: Record<SettingsPage, string> = {
  [GeneralSettingPage.Profile]: "Profile",
  [GeneralSettingPage.Account]: "Account",
  [TeacherSettingPage.Availability]: "Availability",
  [TeacherSettingPage.Topics]: "Topics",
  [TeacherSettingPage.AppointmentTypes]: "Appointment Types",
  [TeacherSettingPage.Locations]: "Locations",
  [AdminSettingPage.Users]: "Users",
  [AdminSettingPage.Classes]: "Classes",
};

export default function SettingsLayout({
  children,
  activePage,
}: {
  children: JSX.Element;
  activePage: SettingsPage;
}) {
  const { user, initialLoadStatus } = useAuth(true);

  const renderMenuLink = (page: SettingsPage) => (
    <li key={page}>
      <Link
        className={classNames({
          "is-active": page == activePage,
        })}
        href={`/settings/${page}`}
      >
        {PAGE_TITLES[page]}
      </Link>
    </li>
  );

  if (
    initialLoadStatus == AsyncStatus.Idle ||
    initialLoadStatus == AsyncStatus.Pending
  ) {
    return (
      <main className="container centered-vertically bulma-loader-mixin"></main>
    );
  }

  return (
    <main className="container">
      <div className="columns" style={{ marginTop: "2rem" }}>
        <aside
          className="menu column is-narrow p-4"
          style={{
            minWidth: "16rem",
            marginTop: "2.5rem",
          }}
        >
          <p className="menu-label">General</p>
          <ul className="menu-list">
            {Object.values(GeneralSettingPage).map(renderMenuLink)}
          </ul>
          {(user?.permission_level == PermissionLevel.Teacher ||
            user?.permission_level == PermissionLevel.Admin) && (
            <>
              <p className="menu-label">Teacher Settings</p>
              <ul className="menu-list">
                {Object.values(TeacherSettingPage).map(renderMenuLink)}
              </ul>
            </>
          )}
          {user?.permission_level == PermissionLevel.Admin && (
            <>
              <p className="menu-label">Admin Settings</p>
              <ul className="menu-list">
                {Object.values(AdminSettingPage).map(renderMenuLink)}
              </ul>
            </>
          )}
        </aside>

        <div className="column">
          <h1 className="title is-2">{PAGE_TITLES[activePage]}</h1>

          {children}
        </div>
      </div>
    </main>
  );
}
