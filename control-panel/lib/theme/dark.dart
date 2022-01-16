import 'package:flutter/material.dart';

const Color _backgroundDark = Color.fromARGB(255, 37, 42, 48);
const Color _backgroundLight = Color.fromARGB(255, 56, 60, 70);

class DarkTheme {
  static ThemeData data() => ThemeData.dark().copyWith(
    backgroundColor: _backgroundLight,
    primaryColorDark: _backgroundDark,
    canvasColor: _backgroundDark,
    listTileTheme: const ListTileThemeData(
      selectedTileColor: Color.fromARGB(255, 32, 191, 210),
      selectedColor: Colors.white,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(7))
      ),
    ),
    cardTheme: const CardTheme(
      color: _backgroundDark,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(7))
      )
    )
  );
}