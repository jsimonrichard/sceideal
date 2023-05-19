import SettingsLayout, {
  AdminSettingPage,
  GeneralSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

export default function ClassesSettings() {
  return <div></div>;
}

ClassesSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={AdminSettingPage.Classes}>
      {page}
    </SettingsLayout>
  );
};
