import SettingsLayout, {
  TeacherSettingPage,
} from "@/components/settingsLayout";
import { ReactElement } from "react";

export default function AvailabiltySettings() {
  return <div></div>;
}

AvailabiltySettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={TeacherSettingPage.Availability}>
      {page}
    </SettingsLayout>
  );
};
