melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e4 g4 a4 g4 c4 c4 b4 c4 \break

  % Line 2
  c4 b4 a4 d4 c4 b2 a4\fermata \break

  % Line 3
  e4 g4 a4 g4 c4 c4 b4 c4 \break

  % Line 4
  c4 b4 a4 d4 c4 b2 a4\fermata \break

  % Line 5
  a4 a4 a4 b4 a4 g4 fis4 g4 \break

  % Line 6
  g4 a4 b4 c4 b4 a2 b4\fermata \break

  % Line 7
  e,4 g4 a4 g4 c4 c4 b4 c4 \break

  % Line 8
  c4 b4 a4 d4 c4 b2 a4\fermata \bar "|."
}
