import SettingsLayout, {
  TeacherSettingPage,
} from "@/components/settingsLayout";
import { ReactElement } from "react";

export default function LocationsSettings() {
  return <div></div>;
}

LocationsSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={TeacherSettingPage.Locations}>
      {page}
    </SettingsLayout>
  );
};
