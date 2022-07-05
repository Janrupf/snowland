import 'package:flutter/material.dart';
import 'package:snowland_control_panel/api/control_panel_api.dart';
import 'package:snowland_control_panel/logger.dart';

const _logger = Logger("startup_view");

class StartupView extends StatefulWidget {
  const StartupView({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _StartupViewState();
}

class _StartupViewState extends State<StartupView> {
  List<int>? _alive;

  @override
  void initState() {
    super.initState();
    _searchHosts();
  }

  @override
  Widget build(BuildContext context) =>
      _alive == null ? _buildLoading(context) : _buildAlive(context);

  void _onAliveListing(List<int> alive) {
    if (mounted) {
      setState(() {
        _alive = alive;
      });
    }
  }

  Widget _buildLoading(BuildContext context) => const Center(
        child: CircularProgressIndicator(),
      );

  Widget _buildAlive(BuildContext context) {
    if (_alive!.isEmpty) {
      return _buildNotStarted(context);
    }

    throw UnimplementedError();
  }

  Widget _buildNotStarted(BuildContext context) => Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              "No snowland host is running!",
              style: Theme.of(context).textTheme.headline2,
              textAlign: TextAlign.center,
            ),
            const SizedBox(
              height: 50,
            ),
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                ElevatedButton(
                  onPressed: _searchHosts,
                  child: const Text("Search again"),
                ),
                const SizedBox(
                  width: 5,
                ),
                ElevatedButton(
                  onPressed: _startNewHost,
                  child: const Text("Start new instance"),
                )
              ],
            )
          ],
        ),
      );

  void _searchHosts() {
    setState(() {
      _alive = null;
    });

    ControlPanelAPI.instance.listAlive().then(_onAliveListing);
  }

  void _startNewHost() {
    ControlPanelAPI.instance.startNewHost().then((value) {
      _logger.debug("Started new host successfully!");
    }, onError: (error) {
      _logger.error("Failed to start new host: $error");
    });
  }
}
