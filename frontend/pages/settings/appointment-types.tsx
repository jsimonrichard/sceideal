import SettingsLayout, {
  TeacherSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

export default function AppointmentTypeSettings() {
  return <div></div>;
}

AppointmentTypeSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={TeacherSettingPage.AppointmentTypes}>
      {page}
    </SettingsLayout>
  );
};
