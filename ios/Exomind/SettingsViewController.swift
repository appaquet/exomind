import UIKit
import Exocore

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

        case (1, 1): // configure extension
            configureExtension()
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

    private func configureExtension() {
        let endpoints = ExocoreClient.store.httpEndpoints()
        let authToken = ExocoreClient.cell.generateAuthToken()

        if let authToken = authToken, let endpoint = endpoints.first {
            ExtensionUtils.setStoreEndpoint(endpoint: endpoint, authToken: authToken)
            let alert = UIAlertController(title: "Success", message: "Endpoint set to \(endpoint)", preferredStyle: UIAlertController.Style.alert)
            alert.addAction(UIAlertAction(title: "Ok", style: .default, handler: nil))
            self.present(alert, animated: true)
        } else {
            let alert = UIAlertController(title: "Error", message: "Couldn't configure extension. Endpoints or auth token missing.", preferredStyle: UIAlertController.Style.alert)
            alert.addAction(UIAlertAction(title: "Ok", style: .default, handler: nil))
            self.present(alert, animated: true)
        }
    }
}
