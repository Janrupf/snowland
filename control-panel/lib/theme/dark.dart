import 'package:flutter/material.dart';

class DarkTheme {
  static ThemeData data() => ThemeData.dark().copyWith(
    backgroundColor: const Color.fromARGB(255, 25, 30, 49),
    primaryColorDark: const Color.fromARGB(255, 25, 30, 49),
    canvasColor: const Color.fromARGB(255, 41, 44, 63),
    listTileTheme: const ListTileThemeData(
      selectedTileColor: Color.fromARGB(255, 16, 206, 173),
      selectedColor: Colors.white,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(7))
      ),
    )
  );
}