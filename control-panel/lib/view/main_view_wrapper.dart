import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/connection_guard.dart';
import 'package:snowland_control_panel/view/connected_view.dart';
import 'package:snowland_control_panel/view/disconnected_view.dart';

class MainViewWrapper extends StatelessWidget {
  const MainViewWrapper({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => IPCConnectionGuard(
        connectedBuilder: _buildConnected,
        erroredBuilder: _erroredBuilder,
        disconnectedBuilder: _buildDisconnected,
      );

  Widget _buildConnected(BuildContext context) => ConnectedView();

  Widget _erroredBuilder(BuildContext context) {
    // TODO:
    throw UnimplementedError();
  }

  Widget _buildDisconnected(BuildContext context) => const DisconnectedView();
}
