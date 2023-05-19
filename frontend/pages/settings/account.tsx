import SettingsLayout, {
  GeneralSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

export default function AccountSettings() {
  return <div></div>;
}

AccountSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={GeneralSettingPage.Account}>
      {page}
    </SettingsLayout>
  );
};
