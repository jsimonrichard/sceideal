import SettingsLayout, { AdminSettingPage } from "@/components/settings_layout";
import { ReactElement } from "react";

export default function MembershipSettings() {
  return <div></div>;
}

MembershipSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={AdminSettingPage.Membership}>
      {page}
    </SettingsLayout>
  );
};
