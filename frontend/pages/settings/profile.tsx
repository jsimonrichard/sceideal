import SettingsLayout, {
  GeneralSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

export default function ProfileSettings() {
  return <div></div>;
}

ProfileSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={GeneralSettingPage.Profile}>
      {page}
    </SettingsLayout>
  );
};
