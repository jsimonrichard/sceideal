import SettingsLayout, {
  AdminSettingPage,
  GeneralSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

export default function UsersSettings() {
  return <div></div>;
}

UsersSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={AdminSettingPage.Users}>{page}</SettingsLayout>
  );
};
