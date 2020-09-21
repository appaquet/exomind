import UIKit

class SettingsViewController: UITableViewController {
    override func viewDidLoad() {
        super.viewDidLoad()

        self.navigationItem.title = "Settings"
    }

    override func viewWillAppear(_ animated: Bool) {
        guard let navCtrl = self.navigationController as? NavigationController else { return }
        navCtrl.setQuickButtonVisibility(false)
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        switch ((indexPath as NSIndexPath).section, (indexPath as NSIndexPath).item) {
        case (0, 0): // edit favorites
            self.exomindBar()
            self.tableView.deselectRow(at: indexPath, animated: false)

        case (1, 0): // cell config
            self.cellConfig()
            self.tableView.deselectRow(at: indexPath, animated: false)

        default:
            print("SettingsViewController > Unhandled setting \(indexPath)")
        }
    }

    private func exomindBar() {
        (self.navigationController as? NavigationController)?.pushObject(.entityId(id: "favorites"))
    }

    private func cellConfig() {
        RootNavigationController.mainInstance()?.showBootstrap(fromRoot: false)
    }
}
