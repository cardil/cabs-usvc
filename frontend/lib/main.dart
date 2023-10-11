import 'package:flutter/material.dart';
import 'package:frontend/home.dart' show HomePage;

void main() {
  runApp(const CabsApp());
}

class CabsApp extends StatelessWidget {
  const CabsApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'μCabs',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.yellow),
        useMaterial3: true,
      ),
      home: const HomePage(title: 'μCabs'),
    );
  }
}
