import UIKit
import Exocore

class SettingsViewController: UITableViewController {
    override func viewDidLoad() {
        super.viewDidLoad()

        self.navigationItem.title = "Settings"
        self.tableView.delegate = self
    }

    override func viewWillAppear(_ animated: Bool) {
        if let nav = self.navigationController as? NavigationController {
            nav.setQuickButtonVisibility(false)
        }
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

        case (1, 2): // logs
            let logs = self.storyboard!.instantiateViewController(withIdentifier: "LogsViewController") as! LogsViewController
            self.navigationController?.pushViewController(logs, animated: true)
            self.tableView.deselectRow(at: indexPath, animated: false)

        default:
            print("SettingsViewController > Unhandled setting \(indexPath)")
        }
    }

    private func exomindBar() {
        (self.navigationController as? NavigationController)?.pushObject(.entityId(id: "favorites"))
    }

    private func cellConfig() {
        RootViewController.mainInstance()?.showBootstrap()
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

    override func tableView(_ tableView: UITableView, titleForFooterInSection section: Int) -> String? {
        if section == 1 {
            let info = Exocore.buildInfo()
            let buildTime = info.buildTime.date
            let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? ""

            return "exomind \(appVersion) (\(buildDate().toLongFormat())) \n exocore \(info.version) (\(buildTime.toLongFormat()))"
        }

        return nil
    }

    override func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        if section == 1 {
            return 100
        }

        return 0
    }

    override func tableView(_ tableView: UITableView, willDisplayFooterView view: UIView, forSection section: Int) {
        let footerView = view as! UITableViewHeaderFooterView
        footerView.textLabel?.textAlignment = .center
    }

}

fileprivate func buildDate() -> Date {
    if let executablePath = Bundle.main.executablePath,
       let attributes = try? FileManager.default.attributesOfItem(atPath: executablePath),
       let date = attributes[.creationDate] as? Date {
        return date
    }

    return Date()
}
