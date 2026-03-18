melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 e2 g4 g4 a2 e2 g4 g4 d2 f2 e2 r2 \break

  % Line 2
  e2 e4 d4 c2 e2 e4 f4 g4 a4 gis2 a2 r2 \break

  % Line 3
  a2 f4 f4 e2 c2 d4 e4 f4 e4 d2 c2 r2 \break

  % Line 4
  e2 g4 a4 b2 g2 a4 c4 b4 b4 a2 r2 \break

  % Line 5
  a2 a4 a4 g2 e2 a4 a4 g2 f2 e2 r2 \break

  % Line 6
  a2 a4 b4 c2 a2 g4 e4 f4 g4 a2 e2 r2 \break

  % Line 7
  e2 f4 d4 e2 c2 d4 e4 f2 d2 e2 c2 \break

  % Line 8
  a'2 g4 g4 a2 c2 c4 b4 a4 g4 f2 e1 \bar "|."
}
