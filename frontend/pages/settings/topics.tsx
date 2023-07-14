import SettingsLayout, {
  GeneralSettingPage,
  TeacherSettingPage,
} from "@/components/settingsLayout";
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
