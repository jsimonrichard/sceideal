import SettingsLayout, {
  GeneralSettingPage,
  TeacherSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

export default function TopicsSettings() {
  return <div></div>;
}

TopicsSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={TeacherSettingPage.Topics}>
      {page}
    </SettingsLayout>
  );
};
