melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 a'4 a4 a4 g4 f2 e4 \break

  % Line 2
  e4 f4 a4 a4 gis4 a2 r2 \break

  % Line 3
  a2 d4 d4 c4 b4 a2 a4 \break

  % Line 4
  g4 f4 d4 f4 e4 d2 r2 \break

  % Line 5
  d2 f4 e4 f4 g4 a2 g2 r2 \break

  % Line 6
  a2 c4 b4 a4 gis4 a2 r2 \break

  % Line 7
  d2 c4 b4 a4 g4 f2 e2 r2 \break

  % Line 8
  d2 g4 f4 e4 e4 d1 \bar "|."
}
